#pragma once

#include <QImageIOHandler>
#include <QImage>

/**
 * @brief Qt image I/O handler for PCX (PC Paintbrush) format
 * 
 * Supports reading PCX images used in Descent 1 & 2 for briefing screens
 * and full-screen images. Handles:
 * - 8-bit indexed color with 256-color palette
 * - 24-bit RGB images
 * - RLE compression
 * 
 * Writing is not supported as PCX is a legacy read-only format.
 */
class PcxHandler : public QImageIOHandler
{
public:
    PcxHandler();
    
    bool canRead() const override;
    bool read(QImage *image) override;
    bool write(const QImage &image) override;
    
    QVariant option(ImageOption option) const override;
    void setOption(ImageOption option, const QVariant &value) override;
    bool supportsOption(ImageOption option) const override;
    
    static bool canRead(QIODevice *device);
    
private:
    bool readHeader();
    bool decompressRle(QByteArray &output, int expectedSize);
    bool readPalette(QByteArray &palette);
    
    struct PcxHeader {
        quint8 manufacturer;
        quint8 version;
        quint8 encoding;
        quint8 bitsPerPixel;
        quint16 xMin, yMin;
        quint16 xMax, yMax;
        quint16 hDpi, vDpi;
        quint8 egaPalette[48];
        quint8 reserved;
        quint8 numPlanes;
        quint16 bytesPerLine;
        quint16 paletteInfo;
        quint16 hScreenSize, vScreenSize;
        quint8 padding[54];
    };
    
    PcxHeader m_header;
    bool m_headerRead;
    QSize m_size;
};
