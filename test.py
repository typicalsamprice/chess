def test(file_one, file_two):
    with open(file_one, "r") as f1:
        with open(file_two, "r") as f2:
            l1s = f1.read().split("\n")
            l2s = f2.read().split("\n")
            for i in range(len(l2s)):
                if len(l1s[i + 1]) == 0:
                    return
                elif l1s[i + 1] != l2s[i]:
                    print("Error: ", l1s[i + 1], " vs ", l2s[i])

test("perft.txt", "perft_mine.txt")
