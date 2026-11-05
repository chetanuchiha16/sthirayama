package internals

import (
	"encoding/json"
	"fmt"
	"io"

	"github.com/hashicorp/raft"
)

type FSM struct {
	engine Engine
}

func (fsm *FSM) Apply(log raft.Log) interface{} {
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

func (fsm *FSM) SnapShot() (raft.FSMSnapshot, error) {
	return nil, nil
}

func (fsm *FSM) Restore(snapshot io.ReadCloser) error {

	return nil
}
