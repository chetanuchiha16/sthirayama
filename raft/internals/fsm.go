package internals

import (
	"encoding/binary"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"raft/pb"

	"github.com/hashicorp/raft"
	"google.golang.org/protobuf/proto"
)

type FSM struct {
	engine MyEngine
}

var _ raft.FSM = (*FSM)(nil)

func (fsm *FSM) Apply(log *raft.Log) any {
	var comm command
	err := json.Unmarshal(log.Data, &comm)
	if err != nil {
		fmt.Println("failed to unmarshal command")
	}
	switch comm.Op {
	case "set":
		fsm.engine.Set(comm.Key, comm.Value)

	case "get":
		fsm.engine.Get(comm.Key)
	case "del":
		fsm.engine.Del(comm.Key)
	}
	return nil
}

func (fsm *FSM) Snapshot() (raft.FSMSnapshot, error) {
	return &MyFSMSnapshot{
		engine: fsm.engine,
	}, nil
}

func (fsm *FSM) Restore(rc io.ReadCloser) error {
	defer rc.Close()
	stream, err := fsm.engine.grpc_client.Restore(fsm.engine.ctx)
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
