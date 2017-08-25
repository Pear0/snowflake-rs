import socket
import struct

HOST='localhost'
PORT=47322

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    s.connect((HOST, PORT))
    

    s.sendall(b'\x50')
    gen_id = struct.unpack('>q', s.recv(8))[0]

print('Received', gen_id)

approx_ms = gen_id >> 22
print('It\'s been approximately {} seconds since the server\'s epoch.'.format(approx_ms / 1000))


