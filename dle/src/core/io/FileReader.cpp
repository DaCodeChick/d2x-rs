#include "FileReader.h"

namespace dle {

FileReader::FileReader()
    : m_file(std::make_unique<QFile>())
    , m_stream(std::make_unique<QDataStream>())
{
}

FileReader::~FileReader() {
    close();
}

bool FileReader::open(const QString& filename) {
    close();
    
    m_file = std::make_unique<QFile>(filename);
    if (!m_file->open(QIODevice::ReadOnly)) {
        return false;
    }
    
    m_stream = std::make_unique<QDataStream>(m_file.get());
    m_stream->setByteOrder(QDataStream::LittleEndian);  // Descent files are little-endian
    
    return true;
}

void FileReader::close() {
    if (m_file && m_file->isOpen()) {
        m_file->close();
    }
}

bool FileReader::isOpen() const {
    return m_file && m_file->isOpen();
}

qint64 FileReader::pos() const {
    return m_file ? m_file->pos() : 0;
}

bool FileReader::seek(qint64 pos) {
    return m_file && m_file->seek(pos);
}

qint64 FileReader::size() const {
    return m_file ? m_file->size() : 0;
}

bool FileReader::atEnd() const {
    return m_stream && m_stream->atEnd();
}

// Read basic types
int8_t FileReader::readInt8() {
    int8_t value;
    m_stream->readRawData(reinterpret_cast<char*>(&value), sizeof(value));
    return value;
}

uint8_t FileReader::readUInt8() {
    uint8_t value;
    m_stream->readRawData(reinterpret_cast<char*>(&value), sizeof(value));
    return value;
}

int16_t FileReader::readInt16() {
    int16_t value;
    *m_stream >> value;
    return value;
}

uint16_t FileReader::readUInt16() {
    uint16_t value;
    *m_stream >> value;
    return value;
}

int32_t FileReader::readInt32() {
    int32_t value;
    *m_stream >> value;
    return value;
}

uint32_t FileReader::readUInt32() {
    uint32_t value;
    *m_stream >> value;
    return value;
}

fix FileReader::readFix() {
    return readInt32();  // Fixed-point is stored as int32
}

void FileReader::readBytes(char* data, qint64 len) {
    m_stream->readRawData(data, len);
}

QByteArray FileReader::readBytes(qint64 len) {
    QByteArray data(len, Qt::Uninitialized);
    m_stream->readRawData(data.data(), len);
    return data;
}

QString FileReader::readString(qint64 maxLen) {
    QByteArray data = readBytes(maxLen);
    // Find null terminator
    int nullPos = data.indexOf('\0');
    if (nullPos >= 0) {
        data.truncate(nullPos);
    }
    return QString::fromLatin1(data);
}

Vector FileReader::readVector() {
    fix x = readFix();
    fix y = readFix();
    fix z = readFix();
    return Vector(x, y, z);
}

Matrix FileReader::readMatrix() {
    Vector right = readVector();
    Vector up = readVector();
    Vector forward = readVector();
    return Matrix(right, up, forward);
}

UVLS FileReader::readUVLS() {
    fix u = readFix();
    fix v = readFix();
    uint16_t light = readUInt16();
    return UVLS(u, v, light);
}

void FileReader::skip(qint64 bytes) {
    seek(pos() + bytes);
}

bool FileReader::hasError() const {
    return m_stream && m_stream->status() != QDataStream::Ok;
}

QString FileReader::errorString() const {
    if (!m_file) {
        return "No file opened";
    }
    if (!m_file->isOpen()) {
        return "File not open";
    }
    if (hasError()) {
        switch (m_stream->status()) {
            case QDataStream::ReadPastEnd:
                return "Read past end of file";
            case QDataStream::ReadCorruptData:
                return "Read corrupt data";
            default:
                return "Unknown error";
        }
    }
    return m_file->errorString();
}

} // namespace dle
