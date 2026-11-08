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
	Ctx        context.Context
	GrpcClient pb.SthirayamaClient
}

func (engine *MyEngine) Get(key []byte) ([]byte, error) {

	getResponse, err := engine.GrpcClient.Get(engine.Ctx, &pb.GetRequest{Key: key})
	if err != nil {
		log.Printf("Error Get %v", err)
		return nil, err
	}
	return getResponse.Val, nil
}

func (engine *MyEngine) Set(key, value []byte) bool {

	resp, err := engine.GrpcClient.Set(engine.Ctx, &pb.SetRequest{Key: key, Val: value})
	if err != nil {
		fmt.Println("error grpc set")
		return false

	}

	return resp.Ok

}

func (engine *MyEngine) Del(key []byte) bool {

	delResponse, err := engine.GrpcClient.Del(engine.Ctx, &pb.DelRequest{Key: key})
	if err != nil {
		log.Printf("Error Del %v", err)
		return false
	}
	return delResponse.Ok
}

func (self *MyEngine) Snapshot() {
}
