package main

import (
	"bytes"
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"raft/internals"
	"raft/pb"
	"syscall"

	"github.com/hashicorp/raft"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func main() {
	nodeId := flag.String("id", "node1", "Unique Raft Node ID")
	engineAddr := flag.String("engine", "127.0.0.1:50051", "Rust gRPC Engine address")
	raftAddr := flag.String("raft", "127.0.0.1:7000", "Raft peer communication address")
	httpAddr := flag.String("http", "127.0.0.1:8000", "HTTP client API address")
	dataDir := flag.String("data", "./data-node1", "Raft snapshot data directory")
	bootstrap := flag.Bool("bootstrap", false, "bootstrap this node as the cluster leader")
	joinAddr := flag.String("join", "", "Existing Raft leader HTTP address to join")
	flag.Parse()
	conn, err := grpc.NewClient(*engineAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("Failed to connect to Rust Engine: %v", err)
	}
	defer conn.Close()

	client := pb.NewSthirayamaClient(conn)

	engine := internals.MyEngine{Ctx: context.Background(), GrpcClient: client}
	fsm := internals.NewFSM(engine)
	store := internals.NewStore(*nodeId, *raftAddr, *dataDir, fsm)
	if err := store.Open(*bootstrap); err != nil {
		log.Fatalf("Failed to open store: %v", err)

	}

	if *joinAddr != "" {
		if err := sendJoinRequest(*joinAddr, *nodeId, *raftAddr); err != nil {
			log.Fatalf("Failed to join the cluster %v", err)
		}
	}

	http.HandleFunc("/set", func(w http.ResponseWriter, r *http.Request) {
		if store.Raft().State() != raft.Leader {
			store.ForwartToLeader(w, r)
		}
		var req struct {
			Key   string `json:"key"`
			Value string `json:"value"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		if err := store.Set([]byte(req.Key), []byte(req.Value)); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		w.Write([]byte("OK\n"))
	})

	http.HandleFunc("/get", func(w http.ResponseWriter, r *http.Request) {
		if store.Raft().State() != raft.Leader {
			store.ForwartToLeader(w, r)
		}
		key := r.URL.Query().Get("key")
		val, err := engine.Get([]byte(key))
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		w.Write(val)
	})

	http.HandleFunc("/del", func(w http.ResponseWriter, r *http.Request) {

		if store.Raft().State() != raft.Leader {
			store.ForwartToLeader(w, r)
		}
		key := r.URL.Query().Get("key")
		if err := store.Del([]byte(key)); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		w.Write([]byte("OK\n"))
	})

	http.HandleFunc("/join", func(w http.ResponseWriter, r *http.Request) {

		if store.Raft().State() != raft.Leader {
			store.ForwartToLeader(w, r)
		}
		var req struct {
			NodeID   string `json:"node_id"`
			RaftAddr string `json:"raft_addr"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		if err := store.Join(req.NodeID, req.RaftAddr); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		w.Write([]byte("Joined\n"))
	})
	go func() {
		log.Printf("HTTP API listening on %s", *httpAddr)
		if err := http.ListenAndServe(*httpAddr, nil); err != nil {
			log.Fatalf("HTTP server error: %v", err)
		}
	}()

	terminate := make(chan os.Signal, 1)
	signal.Notify(terminate, os.Interrupt, syscall.SIGTERM)
	<-terminate
	log.Println("Shutting down raft node...")

}
func sendJoinRequest(leaderHTTP, nodeId, raftAddr string) error {
	payload, _ := json.Marshal(map[string]string{
		"node_id":   nodeId,
		"raft_addr": raftAddr,
	})
	resp, err := http.Post(fmt.Sprintf("http://%s/join", leaderHTTP), "application/json", bytes.NewReader(payload))
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad join response: %d", resp.StatusCode)
	}
	return nil
}
