package internals

import (
	"encoding/json"
	"fmt"
	"log"
	"net"
	"net/http"
	"net/http/httputil"
	"net/url"
	"os"
	"path/filepath"
	"strconv"
	"time"

	"github.com/hashicorp/raft"
	raftboltdb "github.com/hashicorp/raft-boltdb/v2"
)

type command struct {
	Op    string `json:"op,omitempty"`
	Key   []byte `json:"key,omitempty"`
	Value []byte `json:"value,omitempty"`
}
type Store struct {
	nodeId   string
	raftAddr string
	dataDir  string
	raft     *raft.Raft
	fsm      *FSM
}

func (self *Store) Raft() *raft.Raft {
	return self.raft
}

func NewStore(nodeId string, raftAddr, dataDir string, fsm *FSM) *Store {
	return &Store{nodeId: nodeId, raftAddr: raftAddr, dataDir: dataDir, fsm: fsm}
}

func (self *Store) Open(bootstrap bool) error {
	config := raft.DefaultConfig()
	config.LocalID = raft.ServerID(self.nodeId)
	adv, err := net.ResolveTCPAddr("tcp", self.raftAddr)
	transport, err := raft.NewTCPTransport(self.raftAddr, adv, 3, 10*time.Second, os.Stderr)
	if err != nil {
		log.Printf("transport error %v", err)
		return err
	}
	snapStore, err := raft.NewFileSnapshotStore(self.dataDir, 2, os.Stderr)
	boltDB, err := raftboltdb.NewBoltStore(filepath.Join(self.dataDir, "raft.db"))
	if err != nil {
		return err
	}
	defer boltDB.Close()
	logStore := boltDB
	stableStore := boltDB
	ra, err := raft.NewRaft(config, self.fsm, logStore, stableStore, snapStore, transport)
	if err != nil {
		return err
	}
	self.raft = ra
	if bootstrap {
		configuration := raft.Configuration{
			Servers: []raft.Server{
				{
					ID:      config.LocalID,
					Address: transport.LocalAddr(),
				},
			},
		}
		self.raft.BootstrapCluster(configuration)
	}
	return nil
}
func (self *Store) LeaderHTTP() (string, error) {
	leaderRaftAddr := string(self.raft.Leader()) // 700x
	if leaderRaftAddr == "" {
		return "", fmt.Errorf("no leader elected yet")
	}
	host, portStr, err := net.SplitHostPort(leaderRaftAddr)
	if err != nil {
		return "", err
	}
	port, _ := strconv.Atoi(portStr)
	httpPort := port + 1000

	return fmt.Sprintf("%s:%d", host, httpPort), nil
}

func (self *Store) ForwartToLeader(w http.ResponseWriter, r *http.Request) {
	leaderHTTPAddr, err := self.LeaderHTTP()
	if err != nil {
		http.Error(w, "no leader currently available", http.StatusServiceUnavailable)
		return
	}
	targetURL, _ := url.Parse("http://" + leaderHTTPAddr)
	proxy := httputil.NewSingleHostReverseProxy(targetURL)
	log.Printf("Forwarding %s request to leader at %s", r.URL.Path, leaderHTTPAddr)
	proxy.ServeHTTP(w, r)

}

func (self *Store) Set(key, value []byte) error {
	if self.raft.State() != raft.Leader {

		return fmt.Errorf("not a leader %v\n", self.raft.State())
	}
	comm := &command{Op: "set", Key: key, Value: value}
	data, err := json.Marshal(comm)
	if err != nil {
		return err
	}
	applyFuture := self.raft.Apply(data, 10*time.Second)
	return applyFuture.Error()

}

func (self *Store) Del(key []byte) error {
	if self.raft.State() != raft.Leader {
		return fmt.Errorf("not a leader %v\n", self.raft.State())
	}
	comm := &command{Op: "del", Key: key}
	data, err := json.Marshal(comm)
	if err != nil {
		return err
	}
	applyFuture := self.raft.Apply(data, 10*time.Second)
	return applyFuture.Error()

}

func (self *Store) Join(nodeId, addr string) error {
	if self.raft.State() != raft.Leader {
		return fmt.Errorf("not leader")

	}
	future := self.raft.AddVoter(raft.ServerID(nodeId), raft.ServerAddress(addr), 0, 0)
	return future.Error()
}
