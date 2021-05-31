import chess.engine
import json

stockfish = chess.engine.SimpleEngine.popen_uci("stockfish")

board = chess.Board()

openings = {}


def iter_(bo: chess.Board, depth=3):
    if depth == 0:
        return
    val: chess.engine.PovScore = stockfish.analyse(bo, chess.engine.Limit(0.15))["score"]
    value = val.white().score(mate_score=52232)
    openings[bo.fen()] = value
    if abs(value) > 90:
        return
    for m in bo.legal_moves:
        nb = bo.copy()
        nb.push(m)
        iter_(nb, depth - 1)


iter_(chess.Board())

with open("openings.json", "w") as f:
    json.dump(openings, f, separators=(',', ':'))

stockfish.close()
