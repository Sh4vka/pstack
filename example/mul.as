.data
LEN  = [10]
MAS1 = [1, 2, -3, 2, -2, 2, 2, 2, 2, 2]
MAS2 = [1, 2, -3, 2, -2, 2, 2, 2, 2, 2]

.text
PUSH LEN
READ
LDC
PUSH 0        ; sum
PUSH 0        ; i

lp:
DUP
PUSH MAS1
ADD
READ
SWAP
DUP
PUSH MAS2
ADD
READ
ROT
MUL
ROT
ADD
SWAP
INC
LOOP lp

POP
HALT