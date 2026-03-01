#include "PcxHandler.h"
#include <QIODevice>
#include <QDataStream>
#include <QDebug>

PcxHandler::PcxHandler()
    : m_headerRead(false)
{
}

bool PcxHandler::canRead() const
{
    if (!m_headerRead && device()) {
        return canRead(device());
    }
    return m_headerRead;
}

bool PcxHandler::canRead(QIODevice *device)
{
    if (!device) {
        qWarning("PcxHandler::canRead() called with no device");
        return false;
    }
    
    // Check if we can peek at the manufacturer byte
    if (device->isSequential() || device->size() < 128) {
        return false;
    }
    
    // PCX files start with manufacturer byte 0x0A
    char manufacturer;
    if (device->peek(&manufacturer, 1) != 1) {
        return false;
    }
    
    return manufacturer == 0x0A;
}

bool PcxHandler::readHeader()
{
    if (m_headerRead) {
        return true;
    }
    
    QIODevice *dev = device();
    if (!dev) {
        return false;
    }
    
    // Read the 128-byte header
    QByteArray headerData = dev->read(128);
    if (headerData.size() != 128) {
        return false;
    }
    
    const unsigned char *data = reinterpret_cast<const unsigned char*>(headerData.constData());
    
    m_header.manufacturer = data[0];
    m_header.version = data[1];
    m_header.encoding = data[2];
    m_header.bitsPerPixel = data[3];
    m_header.xMin = data[4] | (data[5] << 8);
    m_header.yMin = data[6] | (data[7] << 8);
    m_header.xMax = data[8] | (data[9] << 8);
    m_header.yMax = data[10] | (data[11] << 8);
    m_header.hDpi = data[12] | (data[13] << 8);
    m_header.vDpi = data[14] | (data[15] << 8);
    memcpy(m_header.egaPalette, data + 16, 48);
    m_header.reserved = data[64];
    m_header.numPlanes = data[65];
    m_header.bytesPerLine = data[66] | (data[67] << 8);
    m_header.paletteInfo = data[68] | (data[69] << 8);
    m_header.hScreenSize = data[70] | (data[71] << 8);
    m_header.vScreenSize = data[72] | (data[73] << 8);
    
    // Validate header
    if (m_header.manufacturer != 0x0A) {
        qWarning("PcxHandler: Invalid manufacturer byte: 0x%02X", m_header.manufacturer);
        return false;
    }
    
    if (m_header.encoding != 1) {
        qWarning("PcxHandler: Unsupported encoding: %d", m_header.encoding);
        return false;
    }
    
    // Calculate image dimensions
    int width = m_header.xMax - m_header.xMin + 1;
    int height = m_header.yMax - m_header.yMin + 1;
    
    if (width <= 0 || height <= 0 || width > 32767 || height > 32767) {
        qWarning("PcxHandler: Invalid dimensions: %dx%d", width, height);
        return false;
    }
    
    m_size = QSize(width, height);
    m_headerRead = true;
    
    return true;
}

bool PcxHandler::decompressRle(QByteArray &output, int expectedSize)
{
    QIODevice *dev = device();
    if (!dev) {
        return false;
    }
    
    output.clear();
    output.reserve(expectedSize);
    
    while (output.size() < expectedSize && !dev->atEnd()) {
        char byte;
        if (dev->read(&byte, 1) != 1) {
            return false;
        }
        
        unsigned char ubyte = static_cast<unsigned char>(byte);
        
        // Check if this is a run-length marker
        if ((ubyte & 0xC0) == 0xC0) {
            int count = ubyte & 0x3F;
            if (dev->read(&byte, 1) != 1) {
                return false;
            }
            
            for (int i = 0; i < count && output.size() < expectedSize; i++) {
                output.append(byte);
            }
        } else {
            output.append(byte);
        }
    }
    
    return output.size() == expectedSize;
}

bool PcxHandler::readPalette(QByteArray &palette)
{
    QIODevice *dev = device();
    if (!dev) {
        return false;
    }
    
    // For 8-bit images, palette is at the end of file
    // Marker byte 0x0C followed by 768 bytes (256 colors * 3)
    qint64 palettePos = dev->size() - 769;
    if (palettePos < 128) {
        return false;
    }
    
    if (!dev->seek(palettePos)) {
        return false;
    }
    
    char marker;
    if (dev->read(&marker, 1) != 1 || static_cast<unsigned char>(marker) != 0x0C) {
        return false;
    }
    
    palette = dev->read(768);
    return palette.size() == 768;
}

bool PcxHandler::read(QImage *image)
{
    if (!readHeader()) {
        return false;
    }
    
    QIODevice *dev = device();
    if (!dev) {
        return false;
    }
    
    // Seek back to position 128 (after header)
    if (!dev->seek(128)) {
        return false;
    }
    
    int width = m_size.width();
    int height = m_size.height();
    
    // Handle 8-bit indexed color
    if (m_header.bitsPerPixel == 8 && m_header.numPlanes == 1) {
        // Decompress image data
        int scanlineSize = m_header.bytesPerLine;
        int imageDataSize = scanlineSize * height;
        
        QByteArray imageData;
        if (!decompressRle(imageData, imageDataSize)) {
            qWarning("PcxHandler: Failed to decompress image data");
            return false;
        }
        
        // Read palette
        QByteArray paletteData;
        if (!readPalette(paletteData)) {
            qWarning("PcxHandler: Failed to read palette");
            return false;
        }
        
        // Create indexed image
        *image = QImage(width, height, QImage::Format_Indexed8);
        
        // Set up color table
        QVector<QRgb> colorTable(256);
        const unsigned char *palPtr = reinterpret_cast<const unsigned char*>(paletteData.constData());
        for (int i = 0; i < 256; i++) {
            colorTable[i] = qRgb(palPtr[i * 3], palPtr[i * 3 + 1], palPtr[i * 3 + 2]);
        }
        image->setColorTable(colorTable);
        
        // Copy pixel data
        const unsigned char *srcPtr = reinterpret_cast<const unsigned char*>(imageData.constData());
        for (int y = 0; y < height; y++) {
            unsigned char *destLine = image->scanLine(y);
            memcpy(destLine, srcPtr + y * scanlineSize, width);
        }
        
        return true;
    }
    // Handle 24-bit RGB
    else if (m_header.bitsPerPixel == 8 && m_header.numPlanes == 3) {
        int scanlineSize = m_header.bytesPerLine * 3; // 3 planes
        int imageDataSize = scanlineSize * height;
        
        QByteArray imageData;
        if (!decompressRle(imageData, imageDataSize)) {
            qWarning("PcxHandler: Failed to decompress image data");
            return false;
        }
        
        // Create RGB image
        *image = QImage(width, height, QImage::Format_RGB888);
        
        // PCX stores planes sequentially: RRRR... GGGG... BBBB... per scanline
        const unsigned char *srcPtr = reinterpret_cast<const unsigned char*>(imageData.constData());
        for (int y = 0; y < height; y++) {
            unsigned char *destLine = image->scanLine(y);
            const unsigned char *rPlane = srcPtr + y * scanlineSize;
            const unsigned char *gPlane = rPlane + m_header.bytesPerLine;
            const unsigned char *bPlane = gPlane + m_header.bytesPerLine;
            
            for (int x = 0; x < width; x++) {
                destLine[x * 3 + 0] = rPlane[x];
                destLine[x * 3 + 1] = gPlane[x];
                destLine[x * 3 + 2] = bPlane[x];
            }
        }
        
        return true;
    }
    
    qWarning("PcxHandler: Unsupported format: %d bpp, %d planes", 
             m_header.bitsPerPixel, m_header.numPlanes);
    return false;
}

bool PcxHandler::write(const QImage &image)
{
    Q_UNUSED(image);
    // Writing PCX is not supported
    return false;
}

QVariant PcxHandler::option(ImageOption option) const
{
    if (option == Size && m_headerRead) {
        return m_size;
    }
    return QVariant();
}

void PcxHandler::setOption(ImageOption option, const QVariant &value)
{
    Q_UNUSED(option);
    Q_UNUSED(value);
}

bool PcxHandler::supportsOption(ImageOption option) const
{
    return option == Size;
}
