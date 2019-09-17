import grpc
from prefix_server_pb2_grpc import PublicStub
from prefix_server_pb2 import SearchParams, BlockInterval

# Create channel
channel = grpc.insecure_channel('localhost:8950')
stub = PublicStub(channel)

# Search parameters
interval = BlockInterval(start=123, end=321)
search_params = SearchParams(prefix=b'prefix here', interval=interval)

# Iterate through stream
for item in stub.PrefixSearch(search_params):
    print("Input", item.input_index, "matched prefix in transaction",
          item.raw_tx, "at height", item.block_height)
