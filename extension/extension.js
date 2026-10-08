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
        super({ reactive: false, style_class: 'astro-feed-item' });
        
        let vbox = new St.BoxLayout({ vertical: true, x_expand: true });
        this.add_child(vbox);
        
        let headerBox = new St.BoxLayout({ x_expand: true });
        vbox.add_child(headerBox);
        
        let titleText = `Session ${session.id || 'Unknown'}`;
        if (session.status) {
            titleText += ` • ${session.status}`;
        }
        
        let title = new St.Label({ 
            text: titleText,
            style: 'font-weight: bold;',
            x_expand: true
        });
        title.clutter_text.ellipsize = Pango.EllipsizeMode.END;
        headerBox.add_child(title);
        
        if (session.text) {
            let desc = new St.Label({ 
                text: session.text,
                style: 'font-size: 0.9em; color: #aaaaaa;'
            });
            desc.clutter_text.ellipsize = Pango.EllipsizeMode.END;
            vbox.add_child(desc);
        }
        
        let actionBox = new St.BoxLayout({ style_class: 'astro-action-box' });
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
        
        let silenceBtn = new St.Button({ label: 'Silence', style_class: 'button', can_focus: true });
        silenceBtn.connect('clicked', () => {
            Main.notify('ASTRO', `Silenced session ${session.id}`);
            if (onSilenced) onSilenced();
            this.destroy();
        });
        actionBox.add_child(silenceBtn);
    }
}

class AstroIndicator extends PanelMenu.Button {
    constructor() {
        super(0.0, 'ASTRO Indicator');
        
        this._feedItems = [];
        this.sessionCount = 0;
        this.attentionCount = 0;
        
        // Panel layout
        let box = new St.BoxLayout({ style_class: 'astro-panel-box' });
        
        this.icon = new St.Icon({
            icon_name: 'system-run-symbolic',
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
        let headerItem = new PopupMenu.PopupMenuItem('Agent Sessions', { reactive: false });
        headerItem.label.add_style_class_name('astro-menu-header');
        this.menu.addMenuItem(headerItem);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        this._feedSection = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(this._feedSection);
        
        this._proxy = null;
        this._signalId = 0;
        this._setupDBus();
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
            // Not JSON, wrap it
            session = { id: `msg-${Date.now()}`, text: message, status: 'Active' };
        }
        
        this.sessionCount++;
        if (session.attention) {
            this.attentionCount++;
        }
        
        this._updateBadge();
        this._addEventToFeed(session);
    }
    
    _updateBadge() {
        this.badge.text = `${this.sessionCount}`;
        if (this.attentionCount > 0) {
            this.badge.style_class = 'astro-badge astro-badge-attention';
        } else {
            this.badge.style_class = 'astro-badge';
        }
    }
    
    _addEventToFeed(session) {
        let item = new SessionFeedItem(session, () => {
            // onSilenced
            const idx = this._feedItems.indexOf(item);
            if (idx > -1) {
                this._feedItems.splice(idx, 1);
            }
            if (this.sessionCount > 0) this.sessionCount--;
            if (session.attention && this.attentionCount > 0) this.attentionCount--;
            this._updateBadge();
        });
        
        this._feedSection.addMenuItem(item, 0);
        this._feedItems.unshift(item);
        
        // Truncate
        if (this._feedItems.length > MAX_FEED_ITEMS) {
            let removedItem = this._feedItems.pop();
            removedItem.destroy(); // Destroy it from UI
        }
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
        this._indicator = new AstroIndicator();
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
