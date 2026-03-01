#include "DhfArchive.h"
#include <QDataStream>
#include <QDir>
#include <print>

namespace dle {

std::expected<DhfArchive, DhfError> DhfArchive::open(const QString& path) {
    QFile file(path);
    
    if (!file.open(QIODevice::ReadOnly)) {
        return std::unexpected(DhfError::FileNotFound);
    }
    
    auto result = parseEntries(file);
    if (!result) {
        return std::unexpected(result.error());
    }
    
    return DhfArchive(path, std::move(*result));
}

std::expected<QMap<QString, DhfEntry>, DhfError> DhfArchive::parseEntries(QFile& file) {
    QMap<QString, DhfEntry> entries;
    uint64_t currentOffset = 0;
    
    file.seek(0);
    
    // Check for optional DHF signature
    char sig[3];
    if (file.read(sig, 3) == 3 && memcmp(sig, "DHF", 3) == 0) {
        currentOffset = 3;
    } else {
        file.seek(0);
        currentOffset = 0;
    }
    
    QDataStream stream(&file);
    stream.setByteOrder(QDataStream::LittleEndian);
    
    while (!stream.atEnd()) {
        // Read filename (13 bytes)
        char nameBytes[13];
        if (stream.readRawData(nameBytes, 13) != 13) {
            // EOF reached
            break;
        }
        
        // Parse null-terminated filename
        int nameLen = 0;
        for (int i = 0; i < 13; ++i) {
            if (nameBytes[i] == '\0') {
                nameLen = i;
                break;
            }
        }
        if (nameLen == 0) {
            nameLen = 13;
        }
        
        QString name = QString::fromLatin1(nameBytes, nameLen).toUpper();
        
        // Read size (4 bytes, little-endian)
        uint32_t size;
        stream >> size;
        
        if (stream.status() != QDataStream::Ok) {
            return std::unexpected(DhfError::CorruptEntry);
        }
        
        currentOffset += 17; // 13 + 4
        
        // Store entry
        entries.insert(name, DhfEntry{
            .name = name,
            .offset = currentOffset,
            .size = size
        });
        
        // Skip file data
        if (!file.seek(currentOffset + size)) {
            return std::unexpected(DhfError::IoError);
        }
        currentOffset += size;
    }
    
    return entries;
}

bool DhfArchive::containsFile(const QString& name) const {
    return m_entries.contains(name.toUpper());
}

const DhfEntry* DhfArchive::getEntry(const QString& name) const {
    auto it = m_entries.find(name.toUpper());
    if (it != m_entries.end()) {
        return &it.value();
    }
    return nullptr;
}

std::expected<QByteArray, DhfError> DhfArchive::readFile(const QString& name) {
    const DhfEntry* entry = getEntry(name);
    if (!entry) {
        return std::unexpected(DhfError::EntryNotFound);
    }
    
    QFile file(m_filePath);
    if (!file.open(QIODevice::ReadOnly)) {
        return std::unexpected(DhfError::IoError);
    }
    
    if (!file.seek(entry->offset)) {
        return std::unexpected(DhfError::IoError);
    }
    
    QByteArray buffer = file.read(entry->size);
    if (buffer.size() != static_cast<qint64>(entry->size)) {
        return std::unexpected(DhfError::IoError);
    }
    
    return buffer;
}

std::expected<void, DhfError> DhfArchive::extractFile(const QString& name, const QString& destPath) {
    auto dataResult = readFile(name);
    if (!dataResult) {
        return std::unexpected(dataResult.error());
    }
    
    QFile outFile(destPath);
    if (!outFile.open(QIODevice::WriteOnly)) {
        return std::unexpected(DhfError::IoError);
    }
    
    if (outFile.write(*dataResult) != dataResult->size()) {
        return std::unexpected(DhfError::IoError);
    }
    
    return {};
}

std::expected<void, DhfError> DhfArchive::extractAll(const QString& destDir) {
    QDir dir(destDir);
    if (!dir.exists()) {
        if (!dir.mkpath(".")) {
            return std::unexpected(DhfError::IoError);
        }
    }
    
    for (const auto& entry : m_entries.values()) {
        QString destPath = dir.filePath(entry.name);
        auto result = extractFile(entry.name, destPath);
        if (!result) {
            std::println(stderr, "Failed to extract {}: {}", 
                        entry.name.toStdString(), 
                        dhfErrorString(result.error()).toStdString());
            return std::unexpected(result.error());
        }
        std::println("Extracted: {}", entry.name.toStdString());
    }
    
    return {};
}

} // namespace dle
