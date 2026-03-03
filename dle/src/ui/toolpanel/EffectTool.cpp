#include "EffectTool.h"
#include "ui_EffectTool.h"
#include "core/mine/Mine.h"
#include <QSignalBlocker>

namespace dle {

EffectTool::EffectTool(QWidget* parent)
    : QWidget(parent)
    , ui(std::make_unique<Ui::EffectTool>())
    , m_mine(nullptr)
    , m_selectedObjectId(-1)
{
    ui->setupUi(this);
    setupConnections();
}

EffectTool::~EffectTool() = default;

void EffectTool::setMine(const Mine* mine) {
    m_mine = mine;
    m_selectedObjectId = -1;
    refresh();
}

void EffectTool::refresh() {
    updateObjectList();
    updateEffectControls();
}

void EffectTool::setupConnections() {
    connect(ui->comboObjects, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &EffectTool::onObjectSelected);
    connect(ui->comboParticleType, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &EffectTool::onParticleTypeChanged);
    connect(ui->comboLightningStyle, QOverload<int>::of(&QComboBox::currentIndexChanged),
            this, &EffectTool::onLightningStyleChanged);
    connect(ui->sliderVolume, &QSlider::valueChanged,
            this, &EffectTool::onVolumeChanged);
    
    connect(ui->btnAdd, &QPushButton::clicked, this, &EffectTool::onAdd);
    connect(ui->btnDelete, &QPushButton::clicked, this, &EffectTool::onDelete);
    connect(ui->btnCopy, &QPushButton::clicked, this, &EffectTool::onCopy);
    connect(ui->btnPaste, &QPushButton::clicked, this, &EffectTool::onPaste);
}

void EffectTool::updateObjectList() {
    QSignalBlocker blocker(ui->comboObjects);
    ui->comboObjects->clear();
    
    if (!m_mine) {
        ui->comboObjects->addItem("(no mine loaded)");
        ui->comboObjects->setEnabled(false);
        return;
    }
    
    // Add all objects to the combo box
    int objectCount = m_mine->getObjectCount();
    if (objectCount == 0) {
        ui->comboObjects->addItem("(no objects in mine)");
        ui->comboObjects->setEnabled(false);
        return;
    }
    
    ui->comboObjects->setEnabled(true);
    for (int i = 0; i < objectCount; ++i) {
        const auto& obj = m_mine->getObject(i);
        ui->comboObjects->addItem(QString("Object #%1 (Type %2)").arg(i).arg(static_cast<int>(obj.getType())));
    }
    
    // Select first object if we don't have a selection
    if (m_selectedObjectId < 0 && objectCount > 0) {
        m_selectedObjectId = 0;
        ui->comboObjects->setCurrentIndex(0);
    }
}

void EffectTool::updateEffectControls() {
    // Enable/disable controls based on selection
    bool hasSelection = m_mine && m_selectedObjectId >= 0 && m_selectedObjectId < m_mine->getObjectCount();
    
    ui->tabEffectTypes->setEnabled(hasSelection);
    ui->btnAdd->setEnabled(hasSelection);
    ui->btnDelete->setEnabled(hasSelection);
    ui->btnCopy->setEnabled(hasSelection);
    ui->btnPaste->setEnabled(hasSelection);
    
    if (!hasSelection) {
        return;
    }
    
    // TODO: Load effect data from object when effect system is implemented
    // For now, this is just a UI placeholder
}

void EffectTool::onObjectSelected(int index) {
    if (!m_mine || index < 0 || index >= m_mine->getObjectCount()) {
        m_selectedObjectId = -1;
        updateEffectControls();
        return;
    }
    
    m_selectedObjectId = index;
    updateEffectControls();
}

void EffectTool::onParticleTypeChanged(int /*index*/) {
    // TODO: Update particle effect settings when effect system is implemented
}

void EffectTool::onLightningStyleChanged(int /*index*/) {
    // TODO: Update lightning effect settings when effect system is implemented
}

void EffectTool::onVolumeChanged(int /*value*/) {
    // TODO: Update sound volume when effect system is implemented
}

void EffectTool::onAdd() {
    // TODO: Add effect to object when effect system is implemented
}

void EffectTool::onDelete() {
    // TODO: Remove effect from object when effect system is implemented
}

void EffectTool::onCopy() {
    // TODO: Copy effect settings when effect system is implemented
}

void EffectTool::onPaste() {
    // TODO: Paste effect settings when effect system is implemented
}

} // namespace dle
