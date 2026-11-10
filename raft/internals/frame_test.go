package internals

import (
	"bytes"
	"encoding/binary"
	"io"
	"testing"

	"raft/pb"

	"google.golang.org/protobuf/proto"
)

func TestFrame_TruncatedLengthHeader(t *testing.T) {
	// Only 2 bytes provided instead of 4-byte length prefix
	shortHeader := []byte{0x01, 0x00}
	buf := make([]byte, 4)

	_, err := io.ReadFull(bytes.NewReader(shortHeader), buf)
	if err != io.ErrUnexpectedEOF {
		t.Errorf("Expected io.ErrUnexpectedEOF on 2-byte header, got: %v", err)
	}
}

func TestFrame_TruncatedPayload(t *testing.T) {
	// 4-byte header says payload is 100 bytes, but only 5 bytes are provided
	var header [4]byte
	binary.LittleEndian.PutUint32(header[:], 100)

	shortPayload := []byte("short")
	streamData := append(header[:], shortPayload...)

	r := bytes.NewReader(streamData)

	var lenBuf [4]byte
	_, err := io.ReadFull(r, lenBuf[:])
	if err != nil {
		t.Fatalf("Failed to read header: %v", err)
	}

	length := binary.LittleEndian.Uint32(lenBuf[:])
	if length != 100 {
		t.Fatalf("Expected length 100, got %d", length)
	}

	payloadBuf := make([]byte, length)
	_, err = io.ReadFull(r, payloadBuf)
	if err != io.ErrUnexpectedEOF {
		t.Errorf("Expected io.ErrUnexpectedEOF on truncated payload, got: %v", err)
	}
}

func TestFrame_RoundTripSerialization(t *testing.T) {
	// Verify that a RestoreRequest proto message encodes and decodes with 4-byte framing
	req := &pb.RestoreRequest{
		Key:      []byte("frame_test_key"),
		Value:    []byte("frame_test_val"),
		Sequence: 42,
	}

	data, err := proto.Marshal(req)
	if err != nil {
		t.Fatalf("Failed to marshal RestoreRequest: %v", err)
	}

	var buf bytes.Buffer
	var lenPrefix [4]byte
	binary.LittleEndian.PutUint32(lenPrefix[:], uint32(len(data)))

	buf.Write(lenPrefix[:])
	buf.Write(data)

	// Now decode from the buffer
	var readLen [4]byte
	_, err = io.ReadFull(&buf, readLen[:])
	if err != nil {
		t.Fatalf("Failed to read length prefix: %v", err)
	}

	readLength := binary.LittleEndian.Uint32(readLen[:])
	if readLength != uint32(len(data)) {
		t.Errorf("Length mismatch: got %d, expected %d", readLength, len(data))
	}

	payload := make([]byte, readLength)
	_, err = io.ReadFull(&buf, payload)
	if err != nil {
		t.Fatalf("Failed to read payload: %v", err)
	}

	var decoded pb.RestoreRequest
	if err := proto.Unmarshal(payload, &decoded); err != nil {
		t.Fatalf("Failed to unmarshal decoded payload: %v", err)
	}

	if string(decoded.Key) != "frame_test_key" {
		t.Errorf("Key mismatch: got %s, want frame_test_key", string(decoded.Key))
	}
	if string(decoded.Value) != "frame_test_val" {
		t.Errorf("Value mismatch: got %s, want frame_test_val", string(decoded.Value))
	}
	if decoded.Sequence != 42 {
		t.Errorf("Sequence mismatch: got %d, want 42", decoded.Sequence)
	}
}
