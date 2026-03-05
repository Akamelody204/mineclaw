import subprocess
import json
import sys

def run_test():
    # Start the server
    process = subprocess.Popen(
        [r"target\debug\terminal_server.exe", "--shell", r"C:\Program Files\Git\bin\bash.exe"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=sys.stderr,
        text=True
    )

    # 1. Initialize
    init_req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    }
    print(f"Sending: {json.dumps(init_req)}")
    process.stdin.write(json.dumps(init_req) + "\n")
    process.stdin.flush()
    
    response = process.stdout.readline()
    print(f"Received: {response}")

    # 2. Call tool
    call_req = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "execute_command",
            "arguments": {"command": "ls -la"}
        }
    }
    print(f"Sending: {json.dumps(call_req)}")
    process.stdin.write(json.dumps(call_req) + "\n")
    process.stdin.flush()
    
    response = process.stdout.readline()
    print(f"Received: {response}")

    process.terminate()

if __name__ == "__main__":
    run_test()
