.data
MASS = [3, 10, 20, 30]     // первый элемент: длина

.text
    PUSH MASS              // кладёт адрес MASS в mem (число 0)
    READ                   // читает длину
    LDC
    PUSH 0
    PUSH MASS
    INC                    // адрес первого элемента
lp:
    DUP
    READ
    ROT
    ADD
    SWAP
    INC
    LOOP lp

    SWAP
    HALT
