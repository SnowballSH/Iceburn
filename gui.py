import chess.engine
import chess.pgn

engine1 = chess.engine.SimpleEngine.popen_uci("./target/release/iceburn.exe")
engine2 = chess.engine.SimpleEngine.popen_uci("./target/release/iceburn")

game = chess.pgn.Game()
game.headers["Event"] = "Iceburn Engine vs itself"
game.headers["White"] = game.headers["Black"] = "Iceburn Engine"

node = game

print("calculating...")

board = chess.Board()
while not board.is_game_over():
    result = engine1.play(board, chess.engine.Limit())
    board.push(result.move)
    print(board)
    print()
    node = node.add_variation(result.move)

    if board.is_game_over():
        break

    result = engine2.play(board, chess.engine.Limit())
    board.push(result.move)
    print(board)
    print()
    node = node.add_variation(result.move)

game.headers["Result"] = board.result()

engine1.quit()
engine2.quit()

print(game)
