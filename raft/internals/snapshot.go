package internals

import (
	"encoding/binary"
	"io"
	"log"
	"raft/pb"

	"github.com/hashicorp/raft"
	"google.golang.org/protobuf/proto"
)

type MyFSMSnapshot struct {
	engine MyEngine
}

var _ raft.FSMSnapshot = (*MyFSMSnapshot)(nil)

func (self *MyFSMSnapshot) Persist(sink raft.SnapshotSink) error {
	stream, err := self.engine.GrpcClient.Snapshot(self.engine.Ctx, &pb.SnapshotRequest{})
	if err != nil {
		log.Printf("error snapshot %v", err)
	}
	buf := make([]uint8, 4)
	for {
		resp, err := stream.Recv()

		if err == io.EOF {
			log.Printf("finished streaming %v\n", err)
			break
		}
		if err != nil {
			log.Printf("error stream %v\n", err)
			sink.Cancel()
			return err
		}
		data, err := proto.Marshal(resp)
		if err != nil {
			log.Printf("error marshal %v\n", data)
			sink.Cancel()
			return err
		}

		binary.LittleEndian.PutUint32(buf, uint32(len(data)))

		if _, err = sink.Write(buf); err != nil {
			sink.Cancel()
			return err
		}
		if _, err = sink.Write(data); err != nil {
			sink.Cancel()
			return err
		}
		log.Printf("%v", resp.Key)
	}

	return sink.Close()
}

func (self *MyFSMSnapshot) Release() {

}
