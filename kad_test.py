import random
import subprocess
import sys
import time
from typing import Dict, List, Tuple

BIN_PATH = './target/debug/kademlia.exe'


def run_kad_demo(listen_port, dial_port=None) -> Tuple[str, subprocess.Popen]:
    cmd = [BIN_PATH, str(listen_port)]
    if dial_port is not None:
        cmd += [str(dial_port)]

    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        universal_newlines=True
    )
    peer_id = proc.stdout.readline().strip('\n')

    return peer_id, proc


def format_addr(port) -> str:
    return f"/ip4/127.0.0.1/tcp/{port}"


def format_result(peer_id, addr) -> str:
    return f"{peer_id} : {addr}"


def run(num_peers=10, num_queries=5, start_port=54321, random_connect=True):

    peer_ids: List[str] = []
    addresses: Dict[str, str] = {}
    processes: Dict[str, subprocess.Popen] = {}

    print("Spawning processes...")

    for port in range(start_port, start_port + num_peers):
        if port > start_port:
            if random_connect:
                dial_port = random.randint(start_port, port - 1)
            else:
                dial_port = start_port
        else:
            dial_port = None

        peer_id, proc = run_kad_demo(port, dial_port)
        peer_ids.append(peer_id)
        processes[peer_id] = proc
        addresses[peer_id] = format_addr(port)
        print(f"Node {peer_id} listening on addr {addresses[peer_id]} "
              f"dialing addr {format_addr(dial_port)}")

    queries: Dict[str, List[str]] = {}

    print("Querying for node addresses...")

    for peer_id in peer_ids:
        queries[peer_id] = []
        for _ in range(num_queries):
            query = random.choice(peer_ids)
            queries[peer_id].append(query)
            processes[peer_id].stdin.write(query + '\n')
            print(f"Node {peer_id} querying for address of node {query}")

    print("Sleeping...")
    time.sleep(num_peers)

    print("Checking query outputs...")
    test_ok = True
    for peer_id in peer_ids:
        output, _ = processes[peer_id].communicate(timeout=5)
        for query in queries[peer_id]:
            result = format_result(query, addresses[query])
            if result not in output:
                test_ok = False
                print(f"Result '{result}' not found in output of peer {peer_id}")

    if test_ok:
        print("OK")
    else:
        print("Failed")
        sys.exit(1)


if __name__ == '__main__':
    run(*map(int, sys.argv[1:]))
