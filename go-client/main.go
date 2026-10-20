package main

import (
	"context"
	"go-client/pb"
	"log"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func main() {
	conn, err := grpc.NewClient("127.0.0.1:50051", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()

	client := pb.NewSthirayamaClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*5)

	defer cancel()
	res, err := client.Set(ctx, &pb.SetRequest{Key: "bye", Val: "goodbye"})
	if err != nil {
		log.Fatalf("Error Set %v", err)
	}
	log.Printf("Set %v\n", res)
	getResponse, err := client.Get(ctx, &pb.GetRequest{Key: "bye"})
	if err != nil {
		log.Fatalf("Error Get %v", err)
	}
	log.Printf("Get %v \n", getResponse)
	delResponse, err := client.Del(ctx, &pb.DelRequest{Key: "bye"})
	if err != nil {
		log.Fatalf("Error Del %v", err)
	}

	log.Printf("del %v \n", delResponse)
}
