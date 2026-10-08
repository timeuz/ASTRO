#!/usr/bin/env python3
import sys
import gi

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Adw, Gio, GLib

class AstroPreferencesWindow(Adw.PreferencesWindow):
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.set_title("ASTRO Preferences")
        self.set_search_enabled(True)
        
        # General Page
        page = Adw.PreferencesPage.new()
        page.set_title("General")
        page.set_icon_name("preferences-system-symbolic")
        self.add(page)

        # Behavior Group
        behavior_group = Adw.PreferencesGroup.new()
        behavior_group.set_title("Behavior")
        page.add(behavior_group)

        # Humor Level (ComboRow)
        self.humor_model = Gtk.StringList.new(["Dry", "Normal", "Sassy", "Chaotic"])
        humor_row = Adw.ComboRow.new()
        humor_row.set_title("Humor Level")
        humor_row.set_subtitle("Adjust how snarky ASTRO is")
        humor_row.set_model(self.humor_model)
        humor_row.set_selected(1) # Normal
        behavior_group.add(humor_row)

        # Discreet Mode
        discreet_row = Adw.ActionRow.new()
        discreet_row.set_title("Discreet Mode")
        discreet_row.set_subtitle("Minimize visual intrusion and notifications")
        discreet_switch = Gtk.Switch()
        discreet_switch.set_valign(Gtk.Align.CENTER)
        discreet_row.add_suffix(discreet_switch)
        discreet_row.set_activatable_widget(discreet_switch)
        behavior_group.add(discreet_row)

        # Notification Density
        density_model = Gtk.StringList.new(["Low", "Medium", "High", "All"])
        density_row = Adw.ComboRow.new()
        density_row.set_title("Notification Density")
        density_row.set_subtitle("How often ASTRO alerts you")
        density_row.set_model(density_model)
        density_row.set_selected(1) # Medium
        behavior_group.add(density_row)

        # Shortcuts Group
        shortcuts_group = Adw.PreferencesGroup.new()
        shortcuts_group.set_title("Shortcuts")
        page.add(shortcuts_group)

        shortcut_row = Adw.ActionRow.new()
        shortcut_row.set_title("Global Activation")
        shortcut_row.set_subtitle("Shortcut to trigger ASTRO")
        shortcut_label = Gtk.Label(label="<Super>A")
        shortcut_label.add_css_class("dim-label")
        shortcut_label.set_valign(Gtk.Align.CENTER)
        shortcut_row.add_suffix(shortcut_label)
        shortcuts_group.add(shortcut_row)


class DiagnosticsPage(Adw.Bin):
    def __init__(self):
        super().__init__()
        
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        box.set_margin_top(24)
        box.set_margin_bottom(24)
        box.set_margin_start(24)
        box.set_margin_end(24)
        
        title = Gtk.Label(label="Diagnostics Dashboard")
        title.add_css_class("title-1")
        box.append(title)
        
        subtitle = Gtk.Label(label="Overview of installed hooks and system health")
        subtitle.add_css_class("subtitle-1")
        box.append(subtitle)
        
        # Hooks Group
        hooks_group = Adw.PreferencesGroup.new()
        hooks_group.set_title("Installed Hooks")
        box.append(hooks_group)
        
        hooks = [
            ("Git Hook", "Active", "emblem-ok-symbolic"),
            ("Terminal Hook", "Active", "emblem-ok-symbolic"),
            ("Editor Hook", "Warning", "dialog-warning-symbolic"),
            ("System Monitor", "Active", "emblem-ok-symbolic")
        ]
        
        for name, status, icon in hooks:
            row = Adw.ActionRow.new()
            row.set_title(name)
            row.set_subtitle(f"Status: {status}")
            status_icon = Gtk.Image.new_from_icon_name(icon)
            if status == "Warning":
                status_icon.add_css_class("warning")
            else:
                status_icon.add_css_class("success")
            row.add_suffix(status_icon)
            hooks_group.add(row)
            
        # System Health
        health_group = Adw.PreferencesGroup.new()
        health_group.set_title("System Health")
        
        memory_row = Adw.ActionRow.new()
        memory_row.set_title("Memory Usage")
        memory_label = Gtk.Label(label="45 MB")
        memory_label.set_valign(Gtk.Align.CENTER)
        memory_row.add_suffix(memory_label)
        health_group.add(memory_row)
        
        cpu_row = Adw.ActionRow.new()
        cpu_row.set_title("CPU Usage")
        cpu_label = Gtk.Label(label="0.1%")
        cpu_label.set_valign(Gtk.Align.CENTER)
        cpu_row.add_suffix(cpu_label)
        health_group.add(cpu_row)
        
        box.append(health_group)
        
        scrolled = Gtk.ScrolledWindow()
        scrolled.set_child(box)
        scrolled.set_vexpand(True)
        self.set_child(scrolled)


class OnboardingPage(Adw.Bin):
    def __init__(self, on_complete):
        super().__init__()
        
        status_page = Adw.StatusPage()
        status_page.set_title("Welcome to ASTRO")
        status_page.set_description("Your AI-powered system assistant. Let's get you set up.")
        status_page.set_icon_name("preferences-system-symbolic")
        
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=24)
        box.set_halign(Gtk.Align.CENTER)
        
        # Welcome steps
        steps = [
            "1. Configure your preferences.",
            "2. Install terminal & editor hooks.",
            "3. Ready to assist!"
        ]
        
        for step in steps:
            label = Gtk.Label(label=step)
            label.set_halign(Gtk.Align.START)
            box.append(label)
            
        btn = Gtk.Button(label="Get Started")
        btn.add_css_class("suggested-action")
        btn.add_css_class("pill")
        btn.set_halign(Gtk.Align.CENTER)
        btn.set_size_request(200, -1)
        btn.connect("clicked", lambda x: on_complete())
        
        box.append(btn)
        
        status_page.set_child(box)
        self.set_child(status_page)


class MainWindow(Adw.ApplicationWindow):
    def __init__(self, app):
        super().__init__(application=app)
        self.set_title("ASTRO Settings")
        self.set_default_size(800, 600)
        
        self.main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        self.set_content(self.main_box)
        
        self.header = Adw.HeaderBar()
        self.main_box.append(self.header)
        
        self.menu_button = Gtk.MenuButton()
        self.menu_button.set_icon_name("open-menu-symbolic")
        self.header.pack_end(self.menu_button)
        
        menu_model = Gio.Menu()
        menu_model.append("Preferences", "app.preferences")
        menu_model.append("About ASTRO", "app.about")
        self.menu_button.set_menu_model(menu_model)

        self.stack = Gtk.Stack()
        self.stack.set_transition_type(Gtk.StackTransitionType.CROSSFADE)
        self.main_box.append(self.stack)
        self.stack.set_vexpand(True)
        
        self.onboarding = OnboardingPage(self.finish_onboarding)
        self.stack.add_named(self.onboarding, "onboarding")
        
        self.diagnostics = DiagnosticsPage()
        self.stack.add_named(self.diagnostics, "diagnostics")
        
        self.stack.set_visible_child_name("onboarding")
        
    def finish_onboarding(self):
        self.stack.set_visible_child_name("diagnostics")


class AstroApplication(Adw.Application):
    def __init__(self):
        super().__init__(application_id="com.github.astro.Settings",
                         flags=Gio.ApplicationFlags.FLAGS_NONE)

    def do_activate(self):
        self.win = self.props.active_window
        if not self.win:
            self.win = MainWindow(self)
        self.win.present()
        
    def do_startup(self):
        Adw.Application.do_startup(self)
        
        action = Gio.SimpleAction.new("preferences", None)
        action.connect("activate", self.on_preferences_action)
        self.add_action(action)
        self.set_accels_for_action("app.preferences", ["<Primary>comma"])
        
        about_action = Gio.SimpleAction.new("about", None)
        about_action.connect("activate", self.on_about_action)
        self.add_action(about_action)

    def on_preferences_action(self, action, param):
        pref_win = AstroPreferencesWindow(transient_for=self.win)
        pref_win.present()
        
    def on_about_action(self, action, param):
        about = Adw.AboutWindow.new()
        about.set_transient_for(self.win)
        about.set_application_name("ASTRO Settings")
        about.set_developer_name("ASTRO Team")
        about.set_version("1.0.0")
        about.present()


if __name__ == "__main__":
    app = AstroApplication()
    app.run(sys.argv)
