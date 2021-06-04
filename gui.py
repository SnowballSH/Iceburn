import chess.engine
import chess.pgn

engine1 = chess.engine.SimpleEngine.popen_uci("./target/release/iceburn.exe")
engine2 = chess.engine.SimpleEngine.popen_uci("./target/release/iceburn.exe")

FEN = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

game = chess.pgn.Game()
game.headers["Event"] = "Iceburn Engine vs itself"
game.headers["White"] = "Iceburn Engine"
game.headers["Black"] = "Iceburn Engine"
game.headers["FEN"] = FEN

node = game

print("calculating...")

board = chess.Board(fen=FEN)
game.from_board(board)
while not board.is_game_over():
    result = engine1.play(board, chess.engine.Limit())
    print(result.info)
    board.push(result.move)
    print(board)
    print()
    node = node.add_variation(result.move)

    if board.is_game_over():
        break

    result = engine2.play(board, chess.engine.Limit())
    print(result.info)
    board.push(result.move)
    print(board)
    print()
    node = node.add_variation(result.move)

game.headers["Result"] = board.result()

engine1.quit()
engine2.quit()

print(game)
