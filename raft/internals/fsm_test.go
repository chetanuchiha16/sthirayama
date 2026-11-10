package internals

import (
	"bytes"
	"encoding/json"
	"io"
	"testing"

	"github.com/hashicorp/raft"
)

// MockEngine implements the Engine interface for unit testing without gRPC.
type MockEngine struct {
	data      map[string][]byte
	snapErr   error
	restErr   error
	snapshot  raft.FSMSnapshot
	restored  [][]byte
}

func NewMockEngine() *MockEngine {
	return &MockEngine{
		data: make(map[string][]byte),
	}
}

func (m *MockEngine) Get(key []byte) ([]byte, error) {
	val, ok := m.data[string(key)]
	if !ok {
		return nil, nil
	}
	return val, nil
}

func (m *MockEngine) Set(key, value []byte) bool {
	m.data[string(key)] = value
	return true
}

func (m *MockEngine) Del(key []byte) bool {
	delete(m.data, string(key))
	return true
}

func (m *MockEngine) Snapshot() (raft.FSMSnapshot, error) {
	if m.snapErr != nil {
		return nil, m.snapErr
	}
	return m.snapshot, nil
}

func (m *MockEngine) Restore(rc io.ReadCloser) error {
	defer rc.Close()
	if m.restErr != nil {
		return m.restErr
	}
	buf, err := io.ReadAll(rc)
	if err != nil {
		return err
	}
	m.restored = append(m.restored, buf)
	return nil
}

func TestFSM_Apply_SetAndGet(t *testing.T) {
	mock := NewMockEngine()
	fsm := NewFSM(mock)

	cmdData, err := json.Marshal(command{
		Op:    "set",
		Key:   []byte("cluster_key"),
		Value: []byte("cluster_val"),
	})
	if err != nil {
		t.Fatalf("Failed to marshal command: %v", err)
	}

	raftLog := &raft.Log{
		Index: 1,
		Term:  1,
		Type:  raft.LogCommand,
		Data:  cmdData,
	}

	result := fsm.Apply(raftLog)
	if result != nil {
		t.Errorf("Expected fsm.Apply to return nil, got %v", result)
	}

	val, err := mock.Get([]byte("cluster_key"))
	if err != nil {
		t.Fatalf("Unexpected error on Get: %v", err)
	}
	if string(val) != "cluster_val" {
		t.Errorf("Expected 'cluster_val', got '%s'", string(val))
	}
}

func TestFSM_Apply_Del(t *testing.T) {
	mock := NewMockEngine()
	mock.Set([]byte("key_to_delete"), []byte("temporary_val"))
	fsm := NewFSM(mock)

	cmdData, err := json.Marshal(command{
		Op:  "del",
		Key: []byte("key_to_delete"),
	})
	if err != nil {
		t.Fatalf("Failed to marshal delete command: %v", err)
	}

	raftLog := &raft.Log{
		Index: 2,
		Term:  1,
		Type:  raft.LogCommand,
		Data:  cmdData,
	}

	fsm.Apply(raftLog)

	val, _ := mock.Get([]byte("key_to_delete"))
	if val != nil {
		t.Errorf("Expected key_to_delete to be deleted, but still found value: %s", string(val))
	}
}

func TestFSM_Apply_MalformedJSON(t *testing.T) {
	mock := NewMockEngine()
	fsm := NewFSM(mock)

	// Corrupted / partial JSON payload must not panic
	corruptedLog := &raft.Log{
		Index: 3,
		Term:  1,
		Type:  raft.LogCommand,
		Data:  []byte(`{"op": "set", "key": `), // Malformed JSON
	}

	res := fsm.Apply(corruptedLog)
	if res != nil {
		t.Errorf("Expected nil on malformed JSON, got %v", res)
	}
}

func TestFSM_Apply_UnknownOperation(t *testing.T) {
	mock := NewMockEngine()
	fsm := NewFSM(mock)

	unknownLog := &raft.Log{
		Index: 4,
		Term:  1,
		Type:  raft.LogCommand,
		Data:  []byte(`{"op": "unknown_future_op", "key": "abc"}`),
	}

	// Must handle unknown op gracefully without panic
	res := fsm.Apply(unknownLog)
	if res != nil {
		t.Errorf("Expected nil on unknown op, got %v", res)
	}
}

func TestFSM_RestoreDelegation(t *testing.T) {
	mock := NewMockEngine()
	fsm := NewFSM(mock)

	payload := []byte("mock_snapshot_bytes")
	rc := io.NopCloser(bytes.NewReader(payload))

	if err := fsm.Restore(rc); err != nil {
		t.Fatalf("Expected clean restore delegation, got: %v", err)
	}

	if len(mock.restored) != 1 || string(mock.restored[0]) != string(payload) {
		t.Errorf("Expected mock engine to receive payload '%s', got '%v'", payload, mock.restored)
	}
}
