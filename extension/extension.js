import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import Clutter from 'gi://Clutter';
import Pango from 'gi://Pango';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

const DBUS_SERVICE = 'org.astro.Service';
const DBUS_PATH = '/org/astro/Service';
const DBUS_INTERFACE = 'org.astro.Service';

const AstroInterface = `<node>
  <interface name="${DBUS_INTERFACE}">
    <method name="Ping">
      <arg type="s" name="reply" direction="out" />
    </method>
    <signal name="EventReceived">
      <arg type="s" name="message" />
    </signal>
  </interface>
</node>`;

const AstroProxy = Gio.DBusProxy.makeProxyWrapper(AstroInterface);

const MAX_FEED_ITEMS = 5;

class SessionFeedItem extends PopupMenu.PopupBaseMenuItem {
    constructor(session, onSilenced) {
        super({ reactive: true, style_class: 'astro-feed-item' });
        
        this.session = session;
        this.isExpanded = false;
        
        let vbox = new St.BoxLayout({ vertical: true, x_expand: true });
        this.add_child(vbox);
        
        let headerBox = new St.BoxLayout({ x_expand: true });
        vbox.add_child(headerBox);
        
        let agentName = session.agent_name || 'Gemini CLI';
        let agentColor = agentName === 'Antigravity' ? '#bb86fc' : '#03dac6';
        
        let titleText = `[${agentName}] Session ${session.id || 'Unknown'}`;
        if (session.status) {
            titleText += ` • ${session.status}`;
        }
        
        let title = new St.Label({ 
            text: titleText,
            style: `font-weight: bold; color: ${agentColor};`,
            x_expand: true
        });
        title.clutter_text.ellipsize = Pango.EllipsizeMode.END;
        headerBox.add_child(title);
        
        this.desc = new St.Label({ 
            text: session.text || '',
            style: 'font-size: 0.9em; color: #aaaaaa;'
        });
        this.desc.clutter_text.ellipsize = Pango.EllipsizeMode.END;
        vbox.add_child(this.desc);
        
        this.detailsBox = new St.BoxLayout({ vertical: true, x_expand: true, visible: false });
        let detailsText = `State: ${session.state || 'N/A'}\nEvent: ${session.event_type || 'N/A'}\nTimestamp: ${session.timestamp || 'N/A'}`;
        let detailsLabel = new St.Label({
            text: detailsText,
            style: 'font-size: 0.8em; color: #888888; padding-top: 4px;'
        });
        this.detailsBox.add_child(detailsLabel);
        vbox.add_child(this.detailsBox);
        
        let actionBox = new St.BoxLayout({ style_class: 'astro-action-box', visible: false });
        this.actionBox = actionBox;
        vbox.add_child(actionBox);
        
        // Actions
        let copyBtn = new St.Button({ label: 'Copy ID', style_class: 'button', can_focus: true });
        copyBtn.connect('clicked', () => {
            let clipboard = St.Clipboard.get_default();
            clipboard.set_text(St.ClipboardType.CLIPBOARD, session.id || '');
            Main.notify('ASTRO', `Copied ID: ${session.id}`);
        });
        actionBox.add_child(copyBtn);
        
        if (session.folder) {
            let folderBtn = new St.Button({ label: 'Open Folder', style_class: 'button', can_focus: true });
            folderBtn.connect('clicked', () => {
                try {
                    let file = Gio.File.new_for_path(session.folder);
                    Gio.app_info_launch_default_for_uri(file.get_uri(), null);
                } catch (e) {
                    console.error(`[ASTRO] Failed to open folder: ${e}`);
                }
            });
            actionBox.add_child(folderBtn);
        }
        
        if (agentName === 'Antigravity') {
            let unsupportedLabel = new St.Label({
                text: '(Silence unsupported)',
                style: 'font-size: 0.8em; color: #888888; margin-top: 4px;',
                y_align: Clutter.ActorAlign.CENTER
            });
            actionBox.add_child(unsupportedLabel);
        } else {
            let silenceBtn = new St.Button({ label: 'Silence', style_class: 'button', can_focus: true });
            silenceBtn.connect('clicked', () => {
                Main.notify('ASTRO', `Silenced session ${session.id}`);
                if (onSilenced) onSilenced();
                this.destroy();
            });
            actionBox.add_child(silenceBtn);
        }
        
        this.connect('activate', () => {
            this.isExpanded = !this.isExpanded;
            this.detailsBox.visible = this.isExpanded;
            this.actionBox.visible = this.isExpanded;
            if (this.isExpanded) {
                this.desc.clutter_text.ellipsize = Pango.EllipsizeMode.NONE;
                this.desc.clutter_text.line_wrap = true;
            } else {
                this.desc.clutter_text.ellipsize = Pango.EllipsizeMode.END;
                this.desc.clutter_text.line_wrap = false;
            }
        });
    }
}

class AstroIndicator extends PanelMenu.Button {
    constructor(extDir) {
        super(0.0, 'ASTRO Indicator');
        
        this._feedItems = [];
        this.sessionCount = 0;
        this.attentionCount = 0;
        
        // Panel layout
        let box = new St.BoxLayout({ style_class: 'astro-panel-box' });
        
        let iconPath = extDir + '/icons/astro-symbolic.svg';
        let gicon = Gio.icon_new_for_string(iconPath);
        
        this.icon = new St.Icon({
            gicon: gicon,
            style_class: 'system-status-icon',
            y_align: Clutter.ActorAlign.CENTER,
        });
        box.add_child(this.icon);
        
        this.badge = new St.Label({
            text: '0',
            y_align: Clutter.ActorAlign.CENTER,
            style_class: 'astro-badge'
        });
        box.add_child(this.badge);
        
        this.add_child(box);
        
        // Menu Popover
        let geminiHeaderItem = new PopupMenu.PopupMenuItem('Gemini CLI Sessions', { reactive: false });
        geminiHeaderItem.label.add_style_class_name('astro-menu-header');
        this.menu.addMenuItem(geminiHeaderItem);
        this._geminiSection = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(this._geminiSection);
        
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        let antigravHeaderItem = new PopupMenu.PopupMenuItem('Antigravity Sessions', { reactive: false });
        antigravHeaderItem.label.add_style_class_name('astro-menu-header');
        this.menu.addMenuItem(antigravHeaderItem);
        this._antigravitySection = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(this._antigravitySection);
        
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        this._emptyStateItem = new PopupMenu.PopupMenuItem('Nenhum agente em atividade. Quando comandos forem processados, eles aparecerão aqui.', { reactive: false });
        this._emptyStateItem.label.add_style_class_name('astro-empty-state');
        this._emptyStateItem.label.clutter_text.line_wrap = true;
        this.menu.addMenuItem(this._emptyStateItem);
        
        this._proxy = null;
        this._signalId = 0;
        this._setupDBus();
        this._updateEmptyState();
    }
    
    _updateEmptyState() {
        if (this.sessionCount === 0) {
            this._emptyStateItem.show();
            this._geminiSection.actor.hide();
            this._antigravitySection.actor.hide();
            this.icon.opacity = 178; // ~70% opacity for inactive
        } else {
            this._emptyStateItem.hide();
            this._geminiSection.actor.show();
            this._antigravitySection.actor.show();
            this.icon.opacity = 255;
        }
    }
    
    async _setupDBus() {
        try {
            this._proxy = new AstroProxy(
                Gio.DBus.session,
                DBUS_SERVICE,
                DBUS_PATH,
                (proxy, error) => {
                    if (error) {
                        console.error(`[ASTRO] Failed to connect to D-Bus: ${error.message}`);
                        this._markAllDisconnected();
                        this._updateBadge();
                        return;
                    }
                    console.log('[ASTRO] Connected to D-Bus service');
                    this._updateBadge();
                    
                    this._signalId = this._proxy.connectSignal('EventReceived', (proxy, senderName, [message]) => {
                        this._handleEvent(message);
                    });
                }
            );
        } catch (e) {
            console.error(`[ASTRO] D-Bus setup failed: ${e}`);
        }
    }
    
    _handleEvent(message) {
        let session;
        try {
            session = JSON.parse(message);
        } catch (e) {
            session = { id: `msg-${Date.now()}`, text: message, status: 'Active' };
        }
        
        this.sessionCount++;
        if (session.attention) {
            this.attentionCount++;
        }
        
        this._updateBadge();
        this._addEventToFeed(session);
    }
    
    _markAllDisconnected() {
        for (let item of this._feedItems) {
            item.label = 'Disconnected';
        }
    }
    
    _updateBadge() {
        this.badge.text = `${this.sessionCount}`;
        if (this.attentionCount > 0) {
            this.badge.style_class = 'astro-badge astro-badge-attention';
        } else {
            this.badge.style_class = 'astro-badge';
        }
        this._updateEmptyState();
    }
    
    _addEventToFeed(session) {
        let item = new SessionFeedItem(session, () => {
            const idx = this._feedItems.indexOf(item);
            if (idx > -1) {
                this._feedItems.splice(idx, 1);
            }
            if (this.sessionCount > 0) this.sessionCount--;
            if (session.attention && this.attentionCount > 0) this.attentionCount--;
            this._updateBadge();
        });
        
        let agentName = session.agent_name || 'Gemini CLI';
        if (agentName === 'Antigravity') {
            this._antigravitySection.addMenuItem(item, 0);
        } else {
            this._geminiSection.addMenuItem(item, 0);
        }
        
        this._feedItems.unshift(item);
        
        if (this._feedItems.length > MAX_FEED_ITEMS) {
            let removedItem = this._feedItems.pop();
            removedItem.destroy();
        }
        
        this._updateEmptyState();
    }
    
    destroy() {
        if (this._proxy && this._signalId) {
            this._proxy.disconnectSignal(this._signalId);
        }
        this._proxy = null;
        super.destroy();
    }
}

export default class AstroExtension extends Extension {
    constructor(metadata) {
        super(metadata);
        this._indicator = null;
    }

    enable() {
        console.log('[ASTRO] Extension enabled');
        this._indicator = new AstroIndicator(this.dir.get_path());
        Main.panel.addToStatusArea(this.uuid, this._indicator);
    }

    disable() {
        console.log('[ASTRO] Extension disabled');
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
    }
}
