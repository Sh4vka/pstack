LEN  = 10
MAS1 = [1, 2, -3, 2, -2, 2, 2, 2, 2, 2]
MAS2 = [1, 2, -3, 2, -2, 2, 2, 2, 2, 2]

sum = 0

for i in range(0, LEN):
    sum += MAS1[i] * MAS2[i]

print(sum)