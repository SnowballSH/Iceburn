import argparse
import os

THREADS = 3

GAMES = 20
TIME = 10.0
INC = 0.1

OPENINGS = (
    ("e2e4", "e7e5"),
    ("e2e4", "c7c5"),
    ("e2e4", "e7e6"),
    ("d2d4", "d7d5"),
    ("d2d4", "g8f6"),
)


def run_once(id_: int, q):
    import chess.engine
    import time
    import random

    stat = [0, 0, 0]
    for g in range(GAMES):
        if g % 5 == 0 and g != 0:
            print(f"WIN {stat[0]} DRAW {stat[1]} LOSE {stat[2]}")

        e1 = chess.engine.SimpleEngine.popen_uci("./target/release/iceburn.exe")
        e2 = chess.engine.SimpleEngine.popen_uci("./old/oct10.exe")

        reverse = g % 2 == 1

        if reverse:
            e1, e2 = e2, e1

        times = [TIME, TIME]
        engines = [e1, e2]
        color = 0

        board = chess.Board()

        for m in random.choice(OPENINGS):
            board.push_uci(m)

        draw = 0.0
        while not board.is_game_over():
            now = time.time()
            result = engines[color].play(board, chess.engine.Limit(
                white_clock=times[0], black_clock=times[1],
                white_inc=INC, black_inc=INC,
            ), info=chess.engine.INFO_SCORE)
            # print(f"{['White', 'Black'][color]} played {result.move} score {result.info['score']}")
            # print(f"Times: {times}")
            board.push(result.move)
            times[color] += INC - (time.time() - now)
            if times[color] <= 0.01:
                times[color] = 0.01
            color ^= 1

            if 'score' in result.info:
                if len(board.move_stack) > 80:
                    if -5 < result.info['score'].white().score(mate_score=100000) < 5:
                        draw += 0.5
                    else:
                        draw = 0.0

                if draw >= 10.0:
                    break

        e1.close()
        e2.close()

        print(f"GAME {GAMES * id_ + g + 1} Finished")

        if draw >= 10.0:
            print("Adjunct Draw")
            print("1/2-1/2")
        else:
            print(board.result())

        print(board.fen())
        print()

        if draw >= 10.0 or board.result() == "1/2-1/2":
            stat[1] += 1
        elif board.result() == "1-0":
            if reverse:
                stat[2] += 1
            else:
                stat[0] += 1
        elif board.result() == "0-1":
            if reverse:
                stat[0] += 1
            else:
                stat[2] += 1

    q.put(stat)

    return stat


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description='Build Iceburn')
    parser.add_argument('--test', action='store_const', dest="test", default=False, const=True)
    parser.add_argument('--run', action='store_const', dest="run", default=False, const=True)

    args = parser.parse_args()

    if "RUSTFLAGS" not in os.environ:
        os.environ["RUSTFLAGS"] = ""

    NATIVE = "-C target-cpu=native"

    os.environ["RUSTFLAGS"] = ' '.join((NATIVE,))

    print("NEW RUSTFLAGS: " + repr(os.environ["RUSTFLAGS"]))

    command = "cargo build --release"

    print("building with " + repr(command))

    os.system(command)

    if args.run:
        os.system("cargo run --release")

    os.environ["RUSTFLAGS"] = ""

    print("RESTORED RUSTFLAGS TO " + repr(os.environ["RUSTFLAGS"]))

    if args.test:
        from multiprocessing import Process, Queue

        queues = []

        for i in range(THREADS):
            q = Queue()

            p = Process(target=run_once, args=(i, q))
            p.start()

            queues.append(q)

        stats = [0, 0, 0]

        for q in queues:
            res = q.get(block=True)
            stats[0] += res[0]
            stats[1] += res[1]
            stats[2] += res[2]

        print(f"NEW VERSION: win {stats[0]} draw {stats[1]} lose {stats[2]}")
