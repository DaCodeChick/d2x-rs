#include "ToolPanel.h"
#include "SegmentTool.h"
#include <QTabWidget>
#include <QVBoxLayout>
#include <QLabel>
#include "core/mine/Mine.h"

namespace dle {

// Forward declare stub tool classes
// These will be replaced with real implementations

class WallTool : public QWidget {
public:
    explicit WallTool(QWidget *parent = nullptr) : QWidget(parent) {
        auto *layout = new QVBoxLayout(this);
        layout->addWidget(new QLabel("Wall Tool\n\nConfigure walls, doors, and keys.", this));
        setLayout(layout);
    }
    void setMine(Mine*) {}
    void refresh() {}
};

class TriggerTool : public QWidget {
public:
    explicit TriggerTool(QWidget *parent = nullptr) : QWidget(parent) {
        auto *layout = new QVBoxLayout(this);
        layout->addWidget(new QLabel("Trigger Tool\n\nSet up triggers and targets.", this));
        setLayout(layout);
    }
    void setMine(Mine*) {}
    void refresh() {}
};

class ObjectTool : public QWidget {
public:
    explicit ObjectTool(QWidget *parent = nullptr) : QWidget(parent) {
        auto *layout = new QVBoxLayout(this);
        layout->addWidget(new QLabel("Object Tool\n\nPlace and configure objects.", this));
        setLayout(layout);
    }
    void setMine(Mine*) {}
    void refresh() {}
};

class TextureTool : public QWidget {
public:
    explicit TextureTool(QWidget *parent = nullptr) : QWidget(parent) {
        auto *layout = new QVBoxLayout(this);
        layout->addWidget(new QLabel("Texture Tool\n\nAlign and configure textures.", this));
        setLayout(layout);
    }
    void setMine(Mine*) {}
    void refresh() {}
};

class DiagnosticsTool : public QWidget {
public:
    explicit DiagnosticsTool(QWidget *parent = nullptr) : QWidget(parent) {
        auto *layout = new QVBoxLayout(this);
        layout->addWidget(new QLabel("Diagnostics Tool\n\nLevel validation and statistics.", this));
        setLayout(layout);
    }
    void setMine(Mine*) {}
    void refresh() {}
};

ToolPanel::ToolPanel(QWidget *parent)
    : QDockWidget("Tools", parent)
    , m_tabWidget(nullptr)
    , m_mine(nullptr)
{
    setupUi();
    setupConnections();
}

ToolPanel::~ToolPanel() = default;

void ToolPanel::setupUi() {
    // Create central widget
    auto *centralWidget = new QWidget(this);
    auto *layout = new QVBoxLayout(centralWidget);
    layout->setContentsMargins(0, 0, 0, 0);
    
    // Create tab widget
    m_tabWidget = new QTabWidget(centralWidget);
    layout->addWidget(m_tabWidget);
    
    // Create tool tabs (stubs for now)
    m_segmentTool = std::make_unique<SegmentTool>(m_tabWidget);
    m_wallTool = std::make_unique<WallTool>(m_tabWidget);
    m_triggerTool = std::make_unique<TriggerTool>(m_tabWidget);
    m_objectTool = std::make_unique<ObjectTool>(m_tabWidget);
    m_textureTool = std::make_unique<TextureTool>(m_tabWidget);
    m_diagnosticsTool = std::make_unique<DiagnosticsTool>(m_tabWidget);
    
    // Add tabs
    m_tabWidget->addTab(m_segmentTool.get(), "Segment");
    m_tabWidget->addTab(m_wallTool.get(), "Wall");
    m_tabWidget->addTab(m_triggerTool.get(), "Trigger");
    m_tabWidget->addTab(m_objectTool.get(), "Object");
    m_tabWidget->addTab(m_textureTool.get(), "Texture");
    m_tabWidget->addTab(m_diagnosticsTool.get(), "Diagnostics");
    
    setWidget(centralWidget);
    
    // Set initial size
    setMinimumWidth(250);
    setMaximumWidth(400);
}

void ToolPanel::setupConnections() {
    connect(m_tabWidget, &QTabWidget::currentChanged,
            this, &ToolPanel::onTabChanged);
}

void ToolPanel::setMine(const Mine* mine) {
    m_mine = mine;
    
    // Propagate to all tools
    if (m_segmentTool) m_segmentTool->setMine(mine);
    if (m_wallTool) m_wallTool->setMine(const_cast<Mine*>(mine)); // Stubs take non-const
    if (m_triggerTool) m_triggerTool->setMine(const_cast<Mine*>(mine));
    if (m_objectTool) m_objectTool->setMine(const_cast<Mine*>(mine));
    if (m_textureTool) m_textureTool->setMine(const_cast<Mine*>(mine));
    if (m_diagnosticsTool) m_diagnosticsTool->setMine(const_cast<Mine*>(mine));
    
    refreshAll();
}

void ToolPanel::refreshAll() {
    if (!m_mine) return;
    
    if (m_segmentTool) m_segmentTool->refresh();
    if (m_wallTool) m_wallTool->refresh();
    if (m_triggerTool) m_triggerTool->refresh();
    if (m_objectTool) m_objectTool->refresh();
    if (m_textureTool) m_textureTool->refresh();
    if (m_diagnosticsTool) m_diagnosticsTool->refresh();
}

void ToolPanel::refreshCurrentTab() {
    if (!m_mine || !m_tabWidget) return;
    
    int currentIndex = m_tabWidget->currentIndex();
    switch (currentIndex) {
        case 0: if (m_segmentTool) m_segmentTool->refresh(); break;
        case 1: if (m_wallTool) m_wallTool->refresh(); break;
        case 2: if (m_triggerTool) m_triggerTool->refresh(); break;
        case 3: if (m_objectTool) m_objectTool->refresh(); break;
        case 4: if (m_textureTool) m_textureTool->refresh(); break;
        case 5: if (m_diagnosticsTool) m_diagnosticsTool->refresh(); break;
    }
}

void ToolPanel::showTab(const QString& tabName) {
    if (!m_tabWidget) return;
    
    for (int i = 0; i < m_tabWidget->count(); ++i) {
        if (m_tabWidget->tabText(i) == tabName) {
            m_tabWidget->setCurrentIndex(i);
            break;
        }
    }
}

void ToolPanel::onTabChanged(int index) {
    if (index >= 0 && index < m_tabWidget->count()) {
        QString toolName = m_tabWidget->tabText(index);
        emit toolChanged(toolName);
        refreshCurrentTab();
    }
}

} // namespace dle
