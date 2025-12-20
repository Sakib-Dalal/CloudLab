#!/usr/bin/env python3
"""
CloudLab Dashboard Server
Author: Sakib Dalal
GitHub: https://github.com/Sakib-Dalal
"""

import http.server
import json
import subprocess
import os
import socketserver
import urllib.parse
import socket
import signal
import sys

PORT = int(os.environ.get('CLOUDLAB_PORT', 3000))
CLOUDLAB_DIR = os.path.expanduser('~/.cloudlab')

class Colors:
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    RESET = '\033[0m'
    BOLD = '\033[1m'

def check_port(port):
    """Check if a port is listening"""
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.settimeout(1)
            result = s.connect_ex(('127.0.0.1', port))
            return result == 0
    except:
        return False

def check_process(name):
    """Check if process is running by PID file"""
    pid_file = os.path.join(CLOUDLAB_DIR, 'pids', f'{name}.pid')
    try:
        if not os.path.exists(pid_file):
            return False
        with open(pid_file, 'r') as f:
            pid = int(f.read().strip())
        # Check if process exists
        os.kill(pid, 0)
        return True
    except (ProcessLookupError, ValueError, FileNotFoundError, OSError):
        return False
    except PermissionError:
        # Process exists but we don't have permission
        return True

def check_service(name, port):
    """Check service by both PID and port"""
    pid_running = check_process(name)
    port_open = check_port(port)
    return pid_running or port_open

def get_config():
    """Load configuration"""
    try:
        with open(os.path.join(CLOUDLAB_DIR, 'config.json'), 'r') as f:
            return json.load(f)
    except:
        return {}

def get_system_info():
    """Get system information"""
    info = {
        'cpu_percent': 0,
        'memory_percent': 0,
        'disk_percent': 0,
        'cpu_count': 1,
        'memory_total': 0,
        'disk_total': 0,
        'platform': sys.platform,
        'python_version': sys.version.split()[0]
    }
    
    try:
        import psutil
        info['cpu_percent'] = psutil.cpu_percent(interval=0.1)
        info['memory_percent'] = psutil.virtual_memory().percent
        info['disk_percent'] = psutil.disk_usage('/').percent
        info['cpu_count'] = psutil.cpu_count()
        info['memory_total'] = round(psutil.virtual_memory().total / (1024**3), 1)
        info['disk_total'] = round(psutil.disk_usage('/').total / (1024**3), 1)
    except ImportError:
        pass
    except Exception as e:
        print(f"Error getting system info: {e}")
    
    return info

def get_logs(service, lines=100):
    """Get service logs"""
    log_path = os.path.join(CLOUDLAB_DIR, 'logs', f'{service}.log')
    try:
        with open(log_path, 'r') as f:
            content = f.readlines()[-lines:]
        return ''.join(content)
    except:
        return f'No logs available for {service}'

def list_kernels():
    """List Jupyter kernels"""
    try:
        result = subprocess.run(['cloudlab', 'kernel', 'list'], 
                              capture_output=True, text=True, timeout=30)
        return result.stdout
    except:
        return "Unable to list kernels"

def list_envs():
    """List Python environments"""
    envs = []
    venv = os.path.join(CLOUDLAB_DIR, 'venv')
    if os.path.exists(venv):
        envs.append({'name': 'cloudlab', 'default': True, 'path': venv})
    
    envs_dir = os.path.join(CLOUDLAB_DIR, 'envs')
    if os.path.exists(envs_dir):
        for name in os.listdir(envs_dir):
            path = os.path.join(envs_dir, name)
            if os.path.isdir(path):
                envs.append({'name': name, 'default': False, 'path': path})
    
    return envs

class DashboardHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Suppress default logging
        pass

    def send_json(self, data, status=200):
        body = json.dumps(data).encode('utf-8')
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', len(body))
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()
        self.wfile.write(body)

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.send_header('Content-Length', '0')
        self.end_headers()

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        query = urllib.parse.parse_qs(parsed.query)

        # Serve dashboard HTML
        if path in ['/', '/index.html', '/dashboard.html']:
            html_path = os.path.join(CLOUDLAB_DIR, 'dashboard.html')
            if os.path.exists(html_path):
                with open(html_path, 'rb') as f:
                    content = f.read()
                self.send_response(200)
                self.send_header('Content-Type', 'text/html; charset=utf-8')
                self.send_header('Content-Length', len(content))
                self.end_headers()
                self.wfile.write(content)
            else:
                self.send_json({'error': 'Dashboard HTML not found'}, 404)
            return

        # API: Health check
        if path == '/api/health':
            self.send_json({'status': 'ok', 'version': '1.2.0'})
            return

        # API: Full status
        if path == '/api/status':
            config = get_config()
            tunnel_urls = config.get('tunnel_urls', {})
            
            # Get ports from config
            jupyter_port = config.get('jupyter_port', 8888)
            vscode_port = config.get('vscode_port', 8080)
            ssh_port = config.get('ssh_port', 7681)
            dashboard_port = config.get('dashboard_port', 3000)
            
            self.send_json({
                'jupyter': check_service('jupyter', jupyter_port),
                'vscode': check_service('vscode', vscode_port),
                'ssh': check_service('ssh', ssh_port),
                'dashboard': True,
                'tunnel_jupyter': check_process('tunnel_jupyter'),
                'tunnel_vscode': check_process('tunnel_vscode'),
                'tunnel_ssh': check_process('tunnel_ssh'),
                'tunnel_dashboard': check_process('tunnel_dashboard'),
                'config': config,
                'tunnel_urls': tunnel_urls,
                'system': get_system_info(),
                'kernels': list_kernels(),
                'environments': list_envs()
            })
            return

        # API: Get logs
        if path == '/api/logs':
            service = query.get('service', ['jupyter'])[0]
            lines = int(query.get('lines', ['100'])[0])
            self.send_json({
                'service': service,
                'log': get_logs(service, lines)
            })
            return

        # API: List kernels
        if path == '/api/kernels':
            self.send_json({'kernels': list_kernels()})
            return

        # API: List environments
        if path == '/api/environments':
            self.send_json({'environments': list_envs()})
            return

        # API: Execute cloudlab command
        if path.startswith('/api/command/'):
            cmd_path = path[13:]  # Remove '/api/command/'
            parts = [urllib.parse.unquote(p) for p in cmd_path.split('/') if p]
            
            if parts:
                try:
                    result = subprocess.run(
                        ['cloudlab'] + parts,
                        capture_output=True,
                        text=True,
                        timeout=120
                    )
                    self.send_json({
                        'success': result.returncode == 0,
                        'stdout': result.stdout,
                        'stderr': result.stderr,
                        'command': 'cloudlab ' + ' '.join(parts)
                    })
                except subprocess.TimeoutExpired:
                    self.send_json({
                        'success': False,
                        'error': 'Command timed out after 120 seconds'
                    })
                except FileNotFoundError:
                    self.send_json({
                        'success': False,
                        'error': 'cloudlab command not found in PATH'
                    })
                except Exception as e:
                    self.send_json({
                        'success': False,
                        'error': str(e)
                    })
            else:
                self.send_json({'error': 'No command specified'}, 400)
            return

        # 404 for unknown paths
        self.send_json({'error': 'Not found', 'path': path}, 404)

class ReuseAddrServer(socketserver.TCPServer):
    allow_reuse_address = True

def signal_handler(sig, frame):
    print(f"\n{Colors.YELLOW}Shutting down dashboard server...{Colors.RESET}")
    sys.exit(0)

def main():
    # Handle signals
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    # Ensure directories exist
    os.makedirs(CLOUDLAB_DIR, exist_ok=True)
    os.makedirs(os.path.join(CLOUDLAB_DIR, 'logs'), exist_ok=True)
    os.makedirs(os.path.join(CLOUDLAB_DIR, 'pids'), exist_ok=True)

    # Try to install psutil for system monitoring
    try:
        import psutil
    except ImportError:
        print(f"{Colors.YELLOW}Installing psutil for system monitoring...{Colors.RESET}")
        os.system(f'{sys.executable} -m pip install psutil -q')

    print(f"""
{Colors.CYAN}{Colors.BOLD}☁️  CloudLab Dashboard Server{Colors.RESET}
{Colors.GREEN}URL:{Colors.RESET} http://localhost:{PORT}
{Colors.YELLOW}Press Ctrl+C to stop{Colors.RESET}
""")

    try:
        with ReuseAddrServer(('0.0.0.0', PORT), DashboardHandler) as server:
            server.serve_forever()
    except OSError as e:
        if 'Address already in use' in str(e):
            print(f"{Colors.RED}Error: Port {PORT} is already in use{Colors.RESET}")
            print(f"Try: cloudlab stop dashboard && cloudlab start dashboard")
        else:
            print(f"{Colors.RED}Error: {e}{Colors.RESET}")
        sys.exit(1)

if __name__ == '__main__':
    main()
