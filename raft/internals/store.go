package internals

import (
	"encoding/json"
	"fmt"
	"log"
	"net"
	"os"
	"time"

	"github.com/hashicorp/raft"
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

func NewStore(nodeId string, raftAddr, dataDir string, fsm *FSM) *Store {
	return &Store{nodeId: nodeId, raftAddr: raftAddr, dataDir: dataDir, fsm: fsm}
}

func (self *Store) Open(bootstrap bool) error {
	config := raft.DefaultConfig()
	adv, err := net.ResolveTCPAddr("tcp", self.raftAddr)
	transport, err := raft.NewTCPTransport(self.raftAddr, adv, 3, 10*time.Second, os.Stderr)
	if err != nil {
		log.Printf("transport error %v", err)
		return err
	}
	snapStore, err := raft.NewFileSnapshotStore(self.dataDir, 2, os.Stderr)
	logStore := raft.NewInmemStore()
	stableStore := logStore
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
