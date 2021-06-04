import chess.polyglot
import json

board = chess.Board()

with open("openings.json", "r") as f:
    openings = json.load(f)

with chess.polyglot.open_reader("openings.bin") as reader:
    def iter_(board, depth):
        if board.fen() not in openings:
            try:
                openings[board.fen()] = max(reader.find_all(board), key=lambda x: x.weight).move.uci()
            except:
                pass
        if depth == 0:
            return
        for m in map(lambda x: x.move, reader.find_all(board)):
            board.push(m)
            if board.fen() not in openings:
                try:
                    openings[board.fen()] = max(reader.find_all(board), key=lambda x: x.weight).move.uci()
                except:
                    board.pop()
                    continue
            iter_(board, depth - 1)
            board.pop()

    iter_(board, 17)

with open("openings.json", "w") as f:
    json.dump(openings, f, separators=(',', ':'))
