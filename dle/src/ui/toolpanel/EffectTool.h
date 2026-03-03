#pragma once

#include <QWidget>
#include <memory>

namespace Ui {
class EffectTool;
}

namespace dle {

class Mine;

/**
 * @brief Effect Tool for managing object effects (particles, lightning, sound)
 * 
 * Provides tabbed interface for:
 * - Particle effects (smoke, fire, sparks, etc.)
 * - Lightning effects (bolts, arcs)
 * - Sound effects (ambient sounds, triggers)
 * 
 * Currently a placeholder with UI structure in place.
 * Full effect data structures and rendering will be implemented later.
 */
class EffectTool : public QWidget {
    Q_OBJECT

public:
    explicit EffectTool(QWidget* parent = nullptr);
    ~EffectTool() override;

    /**
     * @brief Set the mine to observe for object/effect data
     * @param mine Pointer to mine (non-owning observation)
     */
    void setMine(const Mine* mine);

public slots:
    /**
     * @brief Refresh all controls to match current mine state
     */
    void refresh();

private slots:
    void onObjectSelected(int index);
    void onParticleTypeChanged(int index);
    void onLightningStyleChanged(int index);
    void onVolumeChanged(int value);
    void onAdd();
    void onDelete();
    void onCopy();
    void onPaste();

private:
    void setupConnections();
    void updateObjectList();
    void updateEffectControls();

    std::unique_ptr<Ui::EffectTool> ui;
    const Mine* m_mine;
    int m_selectedObjectId;
};

} // namespace dle
