# for ce exact array of byte
import struct

life = int(input("life: "))
es = int(input("es: "))

print(struct.pack("<iiiii", life, life, life, es, es).hex())
