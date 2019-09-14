import requests

for i in range(256):
    prefix = "{:02x}".format(i)
    url = "http://35.232.229.28:8083/prefix/" + prefix
    result = requests.get(url)
    if result.content != b'prefix not found':
        print(url)
