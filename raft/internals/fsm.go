package internals

import (
	"encoding/json"
	"fmt"
	"io"

	"github.com/hashicorp/raft"
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
	return nil
}
