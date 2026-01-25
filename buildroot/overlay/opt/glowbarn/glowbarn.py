#!/usr/bin/env python3
"""
GlowBarn OS - Paranormal Research Operating System
Main Application Entry Point

A comprehensive paranormal investigation platform with:
- Real-time sensor monitoring (EMF, temperature, humidity, motion, vibration)
- EVP (Electronic Voice Phenomena) detection and recording
- Night vision camera with motion detection
- Web-based dashboard interface
- Data logging and export
"""

import os
import sys
import yaml
import signal
import logging
import asyncio
from pathlib import Path
from datetime import datetime

# Add our lib path
sys.path.insert(0, '/opt/glowbarn/lib')

__version__ = "1.0.0"
__author__ = "Bad Antics"
__app_name__ = "GlowBarn"

# ASCII Banner
BANNER = r"""
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║     ██████╗ ██╗      ██████╗ ██╗    ██╗██████╗  █████╗ ██████╗   ║
║    ██╔════╝ ██║     ██╔═══██╗██║    ██║██╔══██╗██╔══██╗██╔══██╗  ║
║    ██║  ███╗██║     ██║   ██║██║ █╗ ██║██████╔╝███████║██████╔╝  ║
║    ██║   ██║██║     ██║   ██║██║███╗██║██╔══██╗██╔══██║██╔══██╗  ║
║    ╚██████╔╝███████╗╚██████╔╝╚███╔███╔╝██████╔╝██║  ██║██║  ██║  ║
║     ╚═════╝ ╚══════╝ ╚═════╝  ╚══╝╚══╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝  ║
║                                                                   ║
║                  Paranormal Research Operating System             ║
║                           Version 1.0.0                           ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
"""

# Configuration paths
CONFIG_PATH = Path("/etc/glowbarn/config.yaml")
DATA_PATH = Path("/opt/glowbarn/data")
LOG_PATH = Path("/opt/glowbarn/logs")

# Global state
running = True
config = {}
sensors = {}
web_server = None


def load_config():
    """Load configuration from YAML file."""
    global config
    
    if CONFIG_PATH.exists():
        with open(CONFIG_PATH, 'r') as f:
            config = yaml.safe_load(f)
    else:
        config = {
            'web': {'enabled': True, 'port': 8765, 'host': '0.0.0.0'},
            'sensors': {},
            'audio': {'evp_detection': {'enabled': False}},
            'camera': {'enabled': False},
            'logging': {'level': 'INFO'},
            'alerts': {'enabled': True}
        }
    
    return config


def setup_logging():
    """Configure logging system."""
    LOG_PATH.mkdir(parents=True, exist_ok=True)
    
    log_level = getattr(logging, config.get('logging', {}).get('level', 'INFO'))
    log_file = config.get('logging', {}).get('file', str(LOG_PATH / 'glowbarn.log'))
    
    logging.basicConfig(
        level=log_level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.FileHandler(log_file),
            logging.StreamHandler(sys.stdout)
        ]
    )
    
    return logging.getLogger('glowbarn')


def signal_handler(signum, frame):
    """Handle shutdown signals gracefully."""
    global running
    logging.info(f"Received signal {signum}, shutting down...")
    running = False


class SensorManager:
    """Manages all connected sensors."""
    
    def __init__(self, config):
        self.config = config.get('sensors', {})
        self.sensors = {}
        self.logger = logging.getLogger('sensors')
        
    async def initialize(self):
        """Initialize all configured sensors."""
        self.logger.info("Initializing sensors...")
        
        # EMF Sensor
        if self.config.get('emf', {}).get('enabled'):
            self.logger.info("  - EMF sensor enabled")
            self.sensors['emf'] = {
                'pin': self.config['emf'].get('pin', 17),
                'sample_rate': self.config['emf'].get('sample_rate', 100),
                'value': 0.0
            }
            
        # Temperature Sensor
        if self.config.get('temperature', {}).get('enabled'):
            self.logger.info("  - Temperature sensor enabled")
            self.sensors['temperature'] = {
                'type': self.config['temperature'].get('type', 'ds18b20'),
                'pin': self.config['temperature'].get('pin', 4),
                'value': 0.0
            }
            
        # Humidity Sensor
        if self.config.get('humidity', {}).get('enabled'):
            self.logger.info("  - Humidity sensor enabled")
            self.sensors['humidity'] = {
                'type': self.config['humidity'].get('type', 'dht22'),
                'pin': self.config['humidity'].get('pin', 22),
                'value': 0.0
            }
            
        # Motion Sensor
        if self.config.get('motion', {}).get('enabled'):
            self.logger.info("  - Motion sensor enabled")
            self.sensors['motion'] = {
                'pin': self.config['motion'].get('pin', 27),
                'value': False
            }
            
        # Vibration Sensor
        if self.config.get('vibration', {}).get('enabled'):
            self.logger.info("  - Vibration sensor enabled")
            self.sensors['vibration'] = {
                'pin': self.config['vibration'].get('pin', 23),
                'value': 0.0
            }
            
        # Pressure Sensor
        if self.config.get('pressure', {}).get('enabled'):
            self.logger.info("  - Pressure sensor enabled")
            self.sensors['pressure'] = {
                'type': self.config['pressure'].get('type', 'bmp280'),
                'i2c_address': self.config['pressure'].get('i2c_address', 0x76),
                'value': 0.0
            }
            
        self.logger.info(f"Initialized {len(self.sensors)} sensors")
        
    async def read_all(self):
        """Read current values from all sensors."""
        readings = {
            'timestamp': datetime.now().isoformat(),
            'sensors': {}
        }
        
        for name, sensor in self.sensors.items():
            # Placeholder for actual sensor reading
            # In production, this would read from GPIO/I2C/etc.
            readings['sensors'][name] = {
                'value': sensor.get('value', 0),
                'unit': self._get_unit(name)
            }
            
        return readings
    
    def _get_unit(self, sensor_name):
        """Get the unit for a sensor type."""
        units = {
            'emf': 'mG',
            'temperature': '°F',
            'humidity': '%',
            'motion': 'bool',
            'vibration': 'g',
            'pressure': 'hPa'
        }
        return units.get(sensor_name, '')
        
    async def start_monitoring(self):
        """Start continuous sensor monitoring loop."""
        while running:
            readings = await self.read_all()
            # Store/broadcast readings
            await asyncio.sleep(0.1)  # 10 Hz sampling


class EVPDetector:
    """Electronic Voice Phenomena detection system."""
    
    def __init__(self, config):
        self.config = config.get('audio', {}).get('evp_detection', {})
        self.logger = logging.getLogger('evp')
        self.recording = False
        
    async def initialize(self):
        """Initialize audio capture system."""
        if not self.config.get('enabled'):
            self.logger.info("EVP detection disabled")
            return
            
        self.logger.info("Initializing EVP detection...")
        self.logger.info(f"  - Sample rate: {self.config.get('sample_rate', 44100)}")
        self.logger.info(f"  - Channels: {self.config.get('channels', 2)}")
        
    async def start_detection(self):
        """Start EVP detection loop."""
        if not self.config.get('enabled'):
            return
            
        self.recording = True
        while running and self.recording:
            # Placeholder for audio analysis
            await asyncio.sleep(0.1)


class CameraManager:
    """Camera system with night vision and motion detection."""
    
    def __init__(self, config):
        self.config = config.get('camera', {})
        self.logger = logging.getLogger('camera')
        self.streaming = False
        
    async def initialize(self):
        """Initialize camera system."""
        if not self.config.get('enabled'):
            self.logger.info("Camera disabled")
            return
            
        self.logger.info("Initializing camera...")
        self.logger.info(f"  - Device: {self.config.get('device', '/dev/video0')}")
        self.logger.info(f"  - Resolution: {self.config.get('resolution', '1280x720')}")
        self.logger.info(f"  - Night vision: {self.config.get('night_vision', False)}")
        
    async def start_stream(self):
        """Start video stream."""
        if not self.config.get('enabled'):
            return
            
        self.streaming = True
        while running and self.streaming:
            # Placeholder for frame capture
            await asyncio.sleep(1/30)  # 30 FPS


class WebDashboard:
    """Web-based control and monitoring dashboard."""
    
    def __init__(self, config, sensors, evp, camera):
        self.config = config.get('web', {})
        self.sensors = sensors
        self.evp = evp
        self.camera = camera
        self.logger = logging.getLogger('web')
        self.app = None
        
    async def initialize(self):
        """Initialize web server."""
        if not self.config.get('enabled', True):
            self.logger.info("Web dashboard disabled")
            return
            
        self.logger.info("Initializing web dashboard...")
        self.logger.info(f"  - Host: {self.config.get('host', '0.0.0.0')}")
        self.logger.info(f"  - Port: {self.config.get('port', 8765)}")
        
        try:
            from flask import Flask, jsonify, render_template_string
            from flask_cors import CORS
            
            self.app = Flask(__name__)
            CORS(self.app)
            
            @self.app.route('/')
            def index():
                return render_template_string(self._get_dashboard_html())
                
            @self.app.route('/api/status')
            def status():
                return jsonify({
                    'status': 'running',
                    'version': __version__,
                    'timestamp': datetime.now().isoformat()
                })
                
            @self.app.route('/api/sensors')
            async def sensors():
                readings = await self.sensors.read_all()
                return jsonify(readings)
                
            self.logger.info("Web dashboard initialized")
            
        except ImportError as e:
            self.logger.error(f"Failed to initialize web dashboard: {e}")
            
    def _get_dashboard_html(self):
        """Return the dashboard HTML template."""
        return """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GlowBarn OS - Paranormal Research Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { 
            font-family: 'Segoe UI', sans-serif; 
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
            color: #e0e0e0; 
            min-height: 100vh;
        }
        .header {
            background: rgba(0,0,0,0.3);
            padding: 20px;
            text-align: center;
            border-bottom: 1px solid #00ff88;
        }
        .header h1 { 
            color: #00ff88; 
            font-size: 2.5em;
            text-shadow: 0 0 20px #00ff88;
        }
        .dashboard {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            padding: 20px;
        }
        .card {
            background: rgba(255,255,255,0.05);
            border-radius: 15px;
            padding: 20px;
            border: 1px solid rgba(0,255,136,0.3);
        }
        .card h2 { color: #00ff88; margin-bottom: 15px; }
        .sensor-value { 
            font-size: 2.5em; 
            color: #fff;
            font-weight: bold;
        }
        .status-indicator {
            display: inline-block;
            width: 10px;
            height: 10px;
            border-radius: 50%;
            background: #00ff88;
            animation: pulse 2s infinite;
        }
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>👻 GlowBarn OS</h1>
        <p>Paranormal Research Dashboard <span class="status-indicator"></span></p>
    </div>
    <div class="dashboard">
        <div class="card">
            <h2>📡 EMF Detector</h2>
            <div class="sensor-value" id="emf-value">0.0 mG</div>
        </div>
        <div class="card">
            <h2>🌡️ Temperature</h2>
            <div class="sensor-value" id="temp-value">--°F</div>
        </div>
        <div class="card">
            <h2>💧 Humidity</h2>
            <div class="sensor-value" id="humidity-value">--%</div>
        </div>
        <div class="card">
            <h2>📹 Camera</h2>
            <div id="camera-feed">Camera feed loading...</div>
        </div>
        <div class="card">
            <h2>🎙️ EVP Detection</h2>
            <div id="evp-status">Listening...</div>
        </div>
        <div class="card">
            <h2>📊 Activity Log</h2>
            <div id="activity-log"></div>
        </div>
    </div>
    <script>
        async function updateSensors() {
            try {
                const response = await fetch('/api/sensors');
                const data = await response.json();
                // Update UI with sensor data
            } catch (e) {
                console.error('Failed to fetch sensors:', e);
            }
        }
        setInterval(updateSensors, 1000);
    </script>
</body>
</html>
        """
        
    async def run(self):
        """Run the web server."""
        if not self.app:
            return
            
        # Use asyncio-compatible server
        from werkzeug.serving import make_server
        
        host = self.config.get('host', '0.0.0.0')
        port = self.config.get('port', 8765)
        
        server = make_server(host, port, self.app, threaded=True)
        self.logger.info(f"Web dashboard running at http://{host}:{port}")
        
        # Run in background
        import threading
        thread = threading.Thread(target=server.serve_forever)
        thread.daemon = True
        thread.start()


async def main():
    """Main application entry point."""
    print(BANNER)
    
    # Load configuration
    load_config()
    
    # Setup logging
    logger = setup_logging()
    logger.info(f"Starting GlowBarn OS v{__version__}")
    
    # Register signal handlers
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Initialize components
    sensor_manager = SensorManager(config)
    evp_detector = EVPDetector(config)
    camera_manager = CameraManager(config)
    web_dashboard = WebDashboard(config, sensor_manager, evp_detector, camera_manager)
    
    await sensor_manager.initialize()
    await evp_detector.initialize()
    await camera_manager.initialize()
    await web_dashboard.initialize()
    
    # Start all systems
    logger.info("Starting all systems...")
    
    tasks = [
        asyncio.create_task(sensor_manager.start_monitoring()),
        asyncio.create_task(evp_detector.start_detection()),
        asyncio.create_task(camera_manager.start_stream()),
    ]
    
    # Start web server
    await web_dashboard.run()
    
    logger.info("All systems operational")
    logger.info(f"Web dashboard: http://0.0.0.0:{config.get('web', {}).get('port', 8765)}")
    
    # Wait for shutdown
    while running:
        await asyncio.sleep(1)
    
    # Cleanup
    logger.info("Shutting down...")
    for task in tasks:
        task.cancel()
    
    logger.info("GlowBarn OS shutdown complete")


if __name__ == '__main__':
    asyncio.run(main())
