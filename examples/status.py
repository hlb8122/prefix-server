import grpc
import prefix_server_pb2
from prefix_server_pb2_grpc import PrivateStub
from google.protobuf import empty_pb2
from google.protobuf.json_format import MessageToJson

# Create channel
channel = grpc.insecure_channel('localhost:8951')
stub = PrivateStub(channel)

# Get Status
status = stub.Status(empty_pb2.Empty())
print("State {} and current scrape position {}".format(
    status.state, status.scrape_position))
