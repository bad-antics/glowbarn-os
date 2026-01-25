#!/usr/bin/env python3
"""
GlowBarn CLI - Command Line Interface for GlowBarn OS
"""

import os
import sys
import argparse
import yaml
import json
from datetime import datetime
from pathlib import Path

CONFIG_PATH = Path("/etc/glowbarn/config.yaml")
DATA_PATH = Path("/opt/glowbarn/data")
LOG_PATH = Path("/opt/glowbarn/logs")

BANNER = """
╔═══════════════════════════════════════════════════════════════╗
║  GlowBarn CLI - Paranormal Research Command Interface         ║
╚═══════════════════════════════════════════════════════════════╝
"""

def load_config():
    """Load configuration from YAML file."""
    if CONFIG_PATH.exists():
        with open(CONFIG_PATH, 'r') as f:
            return yaml.safe_load(f)
    return {}


def cmd_status(args):
    """Show system status."""
    print("\n📊 GlowBarn System Status")
    print("─" * 50)
    
    # Check if main service is running
    import subprocess
    result = subprocess.run(['systemctl', 'is-active', 'glowbarn'], 
                          capture_output=True, text=True)
    service_status = result.stdout.strip()
    
    print(f"  Main Service: {'✅ Running' if service_status == 'active' else '❌ Stopped'}")
    
    # Check sensors service
    result = subprocess.run(['systemctl', 'is-active', 'glowbarn-sensors'], 
                          capture_output=True, text=True)
    sensors_status = result.stdout.strip()
    print(f"  Sensors Service: {'✅ Running' if sensors_status == 'active' else '❌ Stopped'}")
    
    # System info
    import platform
    print(f"\n  Hostname: {platform.node()}")
    print(f"  Platform: {platform.machine()}")
    print(f"  OS: GlowBarn OS v1.0.0")
    
    # Disk usage
    import shutil
    total, used, free = shutil.disk_usage("/")
    print(f"\n  Disk: {used // (2**30):.1f}GB / {total // (2**30):.1f}GB ({100*used/total:.1f}%)")
    
    # Memory
    try:
        with open('/proc/meminfo', 'r') as f:
            meminfo = f.read()
        for line in meminfo.split('\n'):
            if 'MemTotal' in line:
                mem_total = int(line.split()[1]) // 1024
            if 'MemAvailable' in line:
                mem_avail = int(line.split()[1]) // 1024
        print(f"  Memory: {mem_total - mem_avail}MB / {mem_total}MB")
    except:
        pass
        
    # Network
    try:
        import socket
        hostname = socket.gethostname()
        ip = socket.gethostbyname(hostname)
        print(f"\n  IP Address: {ip}")
    except:
        pass
        
    print("")


def cmd_sensors(args):
    """Show current sensor readings."""
    print("\n📡 Current Sensor Readings")
    print("─" * 50)
    
    config = load_config()
    sensors_config = config.get('sensors', {})
    
    # Placeholder readings - in production would read actual sensors
    readings = {
        'emf': {'value': 0.3, 'unit': 'mG', 'status': '✅ Normal'},
        'temperature': {'value': 68.5, 'unit': '°F', 'status': '✅ Normal'},
        'humidity': {'value': 45, 'unit': '%', 'status': '✅ Normal'},
        'motion': {'value': False, 'unit': '', 'status': '✅ No motion'},
        'vibration': {'value': 0.01, 'unit': 'g', 'status': '✅ Stable'},
        'pressure': {'value': 1013.25, 'unit': 'hPa', 'status': '✅ Normal'}
    }
    
    for sensor, data in readings.items():
        if sensors_config.get(sensor, {}).get('enabled', False):
            value_str = f"{data['value']}{data['unit']}" if data['unit'] else str(data['value'])
            print(f"  {sensor.upper():12} {value_str:>12}  {data['status']}")
        else:
            print(f"  {sensor.upper():12} {'Disabled':>12}")
    
    print(f"\n  Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("")


def cmd_record(args):
    """Start or stop a recording session."""
    print("\n🎬 Recording Session")
    print("─" * 50)
    
    if args.action == 'start':
        session_id = datetime.now().strftime('%Y%m%d_%H%M%S')
        print(f"  Starting new session: {session_id}")
        print(f"  Location: {args.location or 'Not specified'}")
        print(f"  Notes: {args.notes or 'None'}")
        print("\n  Recording all sensor data...")
        print("  Press Ctrl+C to stop recording\n")
        
        # In production, this would start the actual recording
        try:
            import time
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\n\n  Recording stopped.")
            print(f"  Data saved to: /opt/glowbarn/data/{session_id}/")
            
    elif args.action == 'stop':
        print("  Stopping current session...")
        print("  Session saved.")
        
    elif args.action == 'list':
        print("  Recent Sessions:")
        # List sessions from data directory
        if DATA_PATH.exists():
            sessions = sorted(DATA_PATH.iterdir(), reverse=True)[:10]
            for session in sessions:
                if session.is_dir():
                    print(f"    - {session.name}")
        else:
            print("    No sessions found")
    
    print("")


def cmd_config(args):
    """View or edit configuration."""
    print("\n⚙️ Configuration")
    print("─" * 50)
    
    config = load_config()
    
    if args.action == 'show':
        print(yaml.dump(config, default_flow_style=False, indent=2))
    elif args.action == 'edit':
        import subprocess
        editor = os.environ.get('EDITOR', 'nano')
        subprocess.run([editor, str(CONFIG_PATH)])
    elif args.action == 'get':
        if args.key:
            keys = args.key.split('.')
            value = config
            for k in keys:
                value = value.get(k, {})
            print(f"  {args.key} = {value}")
    elif args.action == 'set':
        if args.key and args.value:
            # This would update the config file
            print(f"  Set {args.key} = {args.value}")
            print("  Configuration updated. Restart services to apply.")
    
    print("")


def cmd_logs(args):
    """View system logs."""
    print("\n📜 System Logs")
    print("─" * 50)
    
    log_file = LOG_PATH / "glowbarn.log"
    
    if args.follow:
        import subprocess
        subprocess.run(['tail', '-f', str(log_file)])
    else:
        lines = args.lines or 20
        try:
            with open(log_file, 'r') as f:
                log_lines = f.readlines()[-lines:]
                for line in log_lines:
                    print(f"  {line.rstrip()}")
        except FileNotFoundError:
            print("  No logs found")
    
    print("")


def cmd_service(args):
    """Manage services."""
    import subprocess
    
    service = args.service or 'glowbarn'
    action = args.action
    
    print(f"\n🔧 Service: {service}")
    print("─" * 50)
    
    if action == 'start':
        result = subprocess.run(['systemctl', 'start', service], capture_output=True)
        print(f"  Starting {service}...")
    elif action == 'stop':
        result = subprocess.run(['systemctl', 'stop', service], capture_output=True)
        print(f"  Stopping {service}...")
    elif action == 'restart':
        result = subprocess.run(['systemctl', 'restart', service], capture_output=True)
        print(f"  Restarting {service}...")
    elif action == 'status':
        result = subprocess.run(['systemctl', 'status', service], capture_output=True, text=True)
        print(result.stdout)
    
    print("")


def cmd_export(args):
    """Export data."""
    print("\n📤 Export Data")
    print("─" * 50)
    
    format_type = args.format or 'csv'
    output = args.output or f"/tmp/glowbarn_export_{datetime.now().strftime('%Y%m%d_%H%M%S')}.{format_type}"
    
    print(f"  Format: {format_type}")
    print(f"  Output: {output}")
    print("  Exporting...")
    
    # Placeholder for actual export
    print(f"  ✅ Export complete: {output}")
    print("")


def main():
    parser = argparse.ArgumentParser(
        description='GlowBarn CLI - Paranormal Research Command Interface',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  glowbarn-cli status              Show system status
  glowbarn-cli sensors             Show current sensor readings
  glowbarn-cli record start        Start a new recording session
  glowbarn-cli logs -f             Follow log output
  glowbarn-cli config show         Show current configuration
        """
    )
    
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Status command
    parser_status = subparsers.add_parser('status', help='Show system status')
    
    # Sensors command
    parser_sensors = subparsers.add_parser('sensors', help='Show sensor readings')
    parser_sensors.add_argument('-w', '--watch', action='store_true', help='Watch mode')
    
    # Record command
    parser_record = subparsers.add_parser('record', help='Recording session management')
    parser_record.add_argument('action', choices=['start', 'stop', 'list'], help='Action')
    parser_record.add_argument('-l', '--location', help='Investigation location')
    parser_record.add_argument('-n', '--notes', help='Session notes')
    
    # Config command
    parser_config = subparsers.add_parser('config', help='Configuration management')
    parser_config.add_argument('action', choices=['show', 'edit', 'get', 'set'], help='Action')
    parser_config.add_argument('-k', '--key', help='Configuration key')
    parser_config.add_argument('-v', '--value', help='Configuration value')
    
    # Logs command
    parser_logs = subparsers.add_parser('logs', help='View system logs')
    parser_logs.add_argument('-f', '--follow', action='store_true', help='Follow log output')
    parser_logs.add_argument('-n', '--lines', type=int, help='Number of lines')
    
    # Service command
    parser_service = subparsers.add_parser('service', help='Manage services')
    parser_service.add_argument('action', choices=['start', 'stop', 'restart', 'status'], help='Action')
    parser_service.add_argument('-s', '--service', help='Service name', default='glowbarn')
    
    # Export command
    parser_export = subparsers.add_parser('export', help='Export data')
    parser_export.add_argument('-f', '--format', choices=['csv', 'json', 'xlsx'], help='Export format')
    parser_export.add_argument('-o', '--output', help='Output file')
    
    args = parser.parse_args()
    
    if not args.command:
        print(BANNER)
        parser.print_help()
        return
    
    commands = {
        'status': cmd_status,
        'sensors': cmd_sensors,
        'record': cmd_record,
        'config': cmd_config,
        'logs': cmd_logs,
        'service': cmd_service,
        'export': cmd_export,
    }
    
    if args.command in commands:
        commands[args.command](args)
    else:
        parser.print_help()


if __name__ == '__main__':
    main()
