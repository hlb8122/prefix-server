import grpc
import prefix_server_pb2
from prefix_server_pb2_grpc import PrivateStub
from prefix_server_pb2 import BlockInterval

# Create channel
channel = grpc.insecure_channel('localhost:8951')
stub = PrivateStub(channel)

# Start Scrape
interval = BlockInterval(start=0, end=3)
stub.Scrape(interval)
