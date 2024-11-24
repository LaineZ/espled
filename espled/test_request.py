import requests

response = requests.post("http://192.168.1.11/set", headers={"Content-Type": "application/octet-stream"}, data=b'\xFF\x00\xFF')

print(f"Status Code: {response.status_code}")
print(f"Response Body: {response.content}")
