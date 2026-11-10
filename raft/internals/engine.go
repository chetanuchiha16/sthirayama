package internals

import (
	"context"
	"encoding/binary"
	"fmt"
	"io"
	"log"
	"raft/pb"

	"github.com/hashicorp/raft"
	"google.golang.org/protobuf/proto"
)

type Engine interface {
	Get(key []byte) ([]byte, error)
	Set(key, value []byte) bool
	Del(key []byte) bool
	Snapshot() (raft.FSMSnapshot, error)
	Restore(rc io.ReadCloser) error
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

// func (self *MyEngine) Snapshot() {
// }
func (self *MyEngine) Snapshot() (raft.FSMSnapshot, error) {
	return &MyFSMSnapshot{
		engine: *self,
	}, nil

}

func (self *MyEngine) Restore(rc io.ReadCloser) error {
	defer rc.Close()
	stream, err := self.GrpcClient.Restore(self.Ctx)
	if err != nil {
		fmt.Printf("error restore %v", err)
		return err
	}
	buf := make([]byte, 4)
	for {
		_, err = io.ReadFull(rc, buf)
		if err == io.EOF {
			log.Printf("sent stream")
			break
		}
		if err != nil {
			log.Printf("error read %v", err)
			return err
		}
		length := binary.LittleEndian.Uint32(buf)
		dataBuff := make([]byte, length)
		if _, err = io.ReadFull(rc, dataBuff); err != nil {
			return err
		}
		var data pb.RestoreRequest
		if err = proto.Unmarshal(dataBuff, &data); err != nil {
			return err
		}
		if err = stream.Send(&data); err != nil {
			return err
		}
	}
	_, err = stream.CloseAndRecv()
	return err
}
