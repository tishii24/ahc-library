import json

if __name__ == "__main__":
    data = []
    for seed in range(10):
        with open(f"tools/in/{seed:04}.txt", "r") as f:
            n, m = map(int, f.readline().split())
        data.append(
            {
                "seed": f"{seed:04}",
                "n": f"{n//20*20}~{n//20*20+20}",
                "m": f"{m//20*20}~{m//20*20+20}",
            }
        )

    json.dump(data, open("input.json", "w"), indent=4)
