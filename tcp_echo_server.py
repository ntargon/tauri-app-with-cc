#!/usr/bin/env python3
"""
TCP Echo Server
クライアントから受信したデータをそのまま送り返すシンプルなTCPエコーサーバー
"""

import socket
import threading
import argparse
import signal
import sys
from datetime import datetime


class TCPEchoServer:
    def __init__(self, host='localhost', port=8080):
        self.host = host
        self.port = port
        self.server_socket = None
        self.running = False
        
    def log(self, message):
        """ログ出力"""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        print(f"[{timestamp}] {message}")
        
    def handle_client(self, client_socket, client_address):
        """クライアント接続を処理"""
        self.log(f"クライアント接続: {client_address}")
        
        try:
            while self.running:
                # データ受信
                data = client_socket.recv(1024)
                if not data:
                    break
                    
                # 受信データをログ出力
                message = data.decode('utf-8', errors='ignore')
                self.log(f"受信 [{client_address}]: {message.strip()}")
                
                # エコー（そのまま送り返す）
                client_socket.send(data)
                self.log(f"送信 [{client_address}]: {message.strip()}")
                
        except Exception as e:
            self.log(f"クライアント処理エラー [{client_address}]: {e}")
        finally:
            client_socket.close()
            self.log(f"クライアント切断: {client_address}")
    
    def signal_handler(self, signum, frame):
        """シグナルハンドラー（Ctrl+C対応）"""
        self.log("終了シグナルを受信しました")
        self.stop()
        
    def start(self):
        """サーバー開始"""
        try:
            # ソケット作成
            self.server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            
            # バインド
            self.server_socket.bind((self.host, self.port))
            self.server_socket.listen(5)
            
            self.running = True
            self.log(f"TCPエコーサーバーを開始しました: {self.host}:{self.port}")
            self.log("クライアントからの接続を待機中...")
            
            # シグナルハンドラー設定
            signal.signal(signal.SIGINT, self.signal_handler)
            signal.signal(signal.SIGTERM, self.signal_handler)
            
            while self.running:
                try:
                    # クライアント接続待機
                    client_socket, client_address = self.server_socket.accept()
                    
                    # 各クライアントを別スレッドで処理
                    client_thread = threading.Thread(
                        target=self.handle_client, 
                        args=(client_socket, client_address)
                    )
                    client_thread.daemon = True
                    client_thread.start()
                    
                except socket.error as e:
                    if self.running:
                        self.log(f"ソケットエラー: {e}")
                    break
                    
        except Exception as e:
            self.log(f"サーバー開始エラー: {e}")
            return False
            
        return True
    
    def stop(self):
        """サーバー停止"""
        self.running = False
        if self.server_socket:
            self.server_socket.close()
        self.log("サーバーを停止しました")


def main():
    """メイン関数"""
    parser = argparse.ArgumentParser(description='TCP Echo Server')
    parser.add_argument(
        '--host', 
        default='localhost', 
        help='バインドするホスト (デフォルト: localhost)'
    )
    parser.add_argument(
        '--port', 
        type=int, 
        default=8080, 
        help='バインドするポート (デフォルト: 8080)'
    )
    
    args = parser.parse_args()
    
    # サーバー作成・開始
    server = TCPEchoServer(args.host, args.port)
    
    try:
        success = server.start()
        if not success:
            sys.exit(1)
    except KeyboardInterrupt:
        server.stop()
    except Exception as e:
        print(f"予期しないエラー: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()