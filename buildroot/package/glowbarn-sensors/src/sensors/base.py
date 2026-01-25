"""
Base classes for GlowBarn sensors.
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from datetime import datetime
from typing import Any, Dict, Optional
import logging


@dataclass
class SensorReading:
    """Represents a single sensor reading."""
    sensor_id: str
    sensor_type: str
    value: Any
    unit: str
    timestamp: datetime
    quality: float = 1.0  # 0.0 to 1.0, indicates reading reliability
    metadata: Optional[Dict[str, Any]] = None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert reading to dictionary."""
        return {
            'sensor_id': self.sensor_id,
            'sensor_type': self.sensor_type,
            'value': self.value,
            'unit': self.unit,
            'timestamp': self.timestamp.isoformat(),
            'quality': self.quality,
            'metadata': self.metadata or {}
        }


class SensorBase(ABC):
    """Abstract base class for all sensors."""
    
    def __init__(self, sensor_id: str, config: Dict[str, Any] = None):
        self.sensor_id = sensor_id
        self.config = config or {}
        self.logger = logging.getLogger(f'sensor.{sensor_id}')
        self._initialized = False
        self._last_reading: Optional[SensorReading] = None
        
    @property
    @abstractmethod
    def sensor_type(self) -> str:
        """Return the type of this sensor."""
        pass
    
    @property
    @abstractmethod
    def unit(self) -> str:
        """Return the unit of measurement for this sensor."""
        pass
    
    @abstractmethod
    async def initialize(self) -> bool:
        """
        Initialize the sensor hardware.
        Returns True if successful, False otherwise.
        """
        pass
    
    @abstractmethod
    async def read(self) -> SensorReading:
        """
        Take a reading from the sensor.
        Returns a SensorReading object.
        """
        pass
    
    async def shutdown(self) -> None:
        """Clean up sensor resources."""
        self._initialized = False
        self.logger.info(f"Sensor {self.sensor_id} shutdown")
    
    @property
    def is_initialized(self) -> bool:
        """Check if sensor is initialized."""
        return self._initialized
    
    @property
    def last_reading(self) -> Optional[SensorReading]:
        """Get the last reading taken."""
        return self._last_reading
    
    def _create_reading(self, value: Any, quality: float = 1.0, 
                       metadata: Dict[str, Any] = None) -> SensorReading:
        """Helper to create a sensor reading."""
        reading = SensorReading(
            sensor_id=self.sensor_id,
            sensor_type=self.sensor_type,
            value=value,
            unit=self.unit,
            timestamp=datetime.now(),
            quality=quality,
            metadata=metadata
        )
        self._last_reading = reading
        return reading


class GPIOSensor(SensorBase):
    """Base class for GPIO-based sensors."""
    
    def __init__(self, sensor_id: str, pin: int, config: Dict[str, Any] = None):
        super().__init__(sensor_id, config)
        self.pin = pin
        self._chip = None
        self._line = None
    
    async def initialize(self) -> bool:
        """Initialize GPIO access."""
        try:
            import gpiod
            self._chip = gpiod.Chip('gpiochip0')
            self._line = self._chip.get_line(self.pin)
            self._initialized = True
            self.logger.info(f"GPIO sensor initialized on pin {self.pin}")
            return True
        except ImportError:
            self.logger.warning("gpiod not available, using mock mode")
            self._initialized = True
            return True
        except Exception as e:
            self.logger.error(f"Failed to initialize GPIO: {e}")
            return False
    
    async def shutdown(self) -> None:
        """Release GPIO resources."""
        if self._line:
            self._line.release()
        if self._chip:
            self._chip.close()
        await super().shutdown()


class I2CSensor(SensorBase):
    """Base class for I2C-based sensors."""
    
    def __init__(self, sensor_id: str, address: int, bus: int = 1, 
                 config: Dict[str, Any] = None):
        super().__init__(sensor_id, config)
        self.address = address
        self.bus = bus
        self._i2c = None
    
    async def initialize(self) -> bool:
        """Initialize I2C access."""
        try:
            import smbus2
            self._i2c = smbus2.SMBus(self.bus)
            self._initialized = True
            self.logger.info(f"I2C sensor initialized at address 0x{self.address:02X}")
            return True
        except ImportError:
            self.logger.warning("smbus2 not available, using mock mode")
            self._initialized = True
            return True
        except Exception as e:
            self.logger.error(f"Failed to initialize I2C: {e}")
            return False
    
    async def shutdown(self) -> None:
        """Release I2C resources."""
        if self._i2c:
            self._i2c.close()
        await super().shutdown()
    
    def read_byte(self, register: int) -> int:
        """Read a single byte from a register."""
        if self._i2c:
            return self._i2c.read_byte_data(self.address, register)
        return 0
    
    def write_byte(self, register: int, value: int) -> None:
        """Write a single byte to a register."""
        if self._i2c:
            self._i2c.write_byte_data(self.address, register, value)


class SerialSensor(SensorBase):
    """Base class for serial/USB sensors."""
    
    def __init__(self, sensor_id: str, port: str, baudrate: int = 9600,
                 config: Dict[str, Any] = None):
        super().__init__(sensor_id, config)
        self.port = port
        self.baudrate = baudrate
        self._serial = None
    
    async def initialize(self) -> bool:
        """Initialize serial port."""
        try:
            import serial
            self._serial = serial.Serial(self.port, self.baudrate, timeout=1)
            self._initialized = True
            self.logger.info(f"Serial sensor initialized on {self.port}")
            return True
        except ImportError:
            self.logger.warning("pyserial not available, using mock mode")
            self._initialized = True
            return True
        except Exception as e:
            self.logger.error(f"Failed to initialize serial: {e}")
            return False
    
    async def shutdown(self) -> None:
        """Release serial resources."""
        if self._serial and self._serial.is_open:
            self._serial.close()
        await super().shutdown()
