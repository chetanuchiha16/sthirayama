package internals

import (
	"bytes"
	"context"
	"io"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"raft/pb"
)

// bufferSnapshotSink simulates HashiCorp raft.SnapshotSink for testing.
type bufferSnapshotSink struct {
	*bytes.Buffer
	canceled bool
}

func newBufferSnapshotSink() *bufferSnapshotSink {
	return &bufferSnapshotSink{Buffer: new(bytes.Buffer)}
}

func (b *bufferSnapshotSink) ID() string {
	return "test-snapshot"
}

func (b *bufferSnapshotSink) Cancel() error {
	b.canceled = true
	return nil
}

func (b *bufferSnapshotSink) Close() error {
	return nil
}

func TestSnapshotAndRestore(t *testing.T) {
	// Connects to local Rust engine gRPC server
	conn, err := grpc.NewClient("127.0.0.1:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to connect to gRPC server at 127.0.0.1:50051: %v", err)
	}
	defer conn.Close()

	client := pb.NewSthirayamaClient(conn)
	ctx := context.Background()

	// 1. Populate test data in Rust engine
	testKVs := map[string]string{
		"user:100": "Alice",
		"user:101": "Bob",
		"user:102": "Charlie",
		"user:103": "David",
	}

	for k, v := range testKVs {
		resp, err := client.Set(ctx, &pb.SetRequest{Key: []byte(k), Val: []byte(v)})
		if err != nil || !resp.Ok {
			t.Fatalf("Failed to set key %s: %v", k, err)
		}
	}

	// 2. Persist snapshot using MyFSMSnapshot into buffer sink
	engine := MyEngine{ctx: ctx, grpc_client: client}
	fsmSnap := &MyFSMSnapshot{engine: engine}
	sink := newBufferSnapshotSink()

	if err := fsmSnap.Persist(sink); err != nil {
		t.Fatalf("Persist snapshot failed: %v", err)
	}

	if sink.Len() == 0 {
		t.Fatalf("Snapshot sink buffer is empty, expected serialized data")
	}
	t.Logf("Snapshot successfully serialized %d bytes into sink", sink.Len())

	// 3. Test Restore via FSM.Restore()
	fsm := &FSM{engine: engine}
	readCloser := io.NopCloser(bytes.NewReader(sink.Bytes()))

	if err := fsm.Restore(readCloser); err != nil {
		t.Fatalf("FSM Restore failed: %v", err)
	}

	// 4. Verify all keys exist and match after restore
	for k, expectedVal := range testKVs {
		val, err := engine.Get([]byte(k))
		if err != nil {
			t.Fatalf("Get failed for key %s: %v", k, err)
		}
		if string(val) != expectedVal {
			t.Fatalf("Value mismatch for key %s: got %s, want %s", k, string(val), expectedVal)
		}
	}

	t.Log("Snapshot and Restore round-trip completed and verified successfully!")
}
