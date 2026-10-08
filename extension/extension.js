import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import Clutter from 'gi://Clutter';
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

class AstroIndicator extends PanelMenu.Button {
    constructor() {
        super(0.0, 'ASTRO Indicator');
        
        // Panel badge/label
        this.label = new St.Label({
            text: 'ASTRO: Init',
            y_align: Clutter.ActorAlign.CENTER,
        });
        this.add_child(this.label);
        
        // Popover for events (chronological feed spike)
        this.menu.addMenuItem(new PopupMenu.PopupMenuItem('Session Feed:', { reactive: false }));
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
                        this.label.text = 'ASTRO: D-Bus Error';
                        return;
                    }
                    console.log('[ASTRO] Connected to D-Bus service');
                    this.label.text = 'ASTRO: Connected';
                    
                    this._signalId = this._proxy.connectSignal('EventReceived', (proxy, senderName, [message]) => {
                        console.log(`[ASTRO] Event received: ${message}`);
                        this.label.text = `ASTRO: Event!`;
                        this._addEventToFeed(message);
                    });
                }
            );
        } catch (e) {
            console.error(`[ASTRO] D-Bus setup failed: ${e}`);
        }
    }
    
    _addEventToFeed(message) {
        const item = new PopupMenu.PopupMenuItem(message);
        // Add to top of feed
        this._feedSection.addMenuItem(item, 0);
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
