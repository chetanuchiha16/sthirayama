package internals

import (
	"context"
	"fmt"
	"log"
	"raft/pb"
)

type Engine interface {
	Get(key []byte) ([]byte, error)
	Set(key, value []byte) bool
	Del(key []byte) bool
	Snapshot()
}

var _ Engine = (*MyEngine)(nil)

type MyEngine struct {
	ctx         context.Context
	grpc_client pb.SthirayamaClient
}

func (engine *MyEngine) Get(key []byte) ([]byte, error) {

	getResponse, err := engine.grpc_client.Get(engine.ctx, &pb.GetRequest{Key: key})
	if err != nil {
		log.Fatalf("Error Get %v", err)
	}
	return []byte(getResponse.Val), nil
}

func (engine *MyEngine) Set(key, value []byte) bool {

	resp, err := engine.grpc_client.Set(engine.ctx, &pb.SetRequest{Key: key, Val: value})
	if err != nil {
		fmt.Println("error grpc set")

	}

	return resp.Ok

}

func (engine *MyEngine) Del(key []byte) bool {

	delResponse, err := engine.grpc_client.Del(engine.ctx, &pb.DelRequest{Key: key})
	if err != nil {
		log.Fatalf("Error Del %v", err)
	}
	return delResponse.Ok
}

func (self *MyEngine) Snapshot() {
}
