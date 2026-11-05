package internals

import (
	"github.com/hashicorp/raft"
)

type command struct {
	Op    string `json:"op,omitempty"`
	Key   []byte `json:"key,omitempty"`
	Value []byte `json:"value,omitempty"`
}
type Store struct {
	fsm  *FSM
	raft *raft.Raft
}
