use std::{
    io::{Read, Write},
    mem,
    net::TcpStream,
    path::{Path, PathBuf},
    process::Command,
    ptr,
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use serde::Deserialize;
use windows_sys::Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::Gdi::{
        AddFontResourceExW, BeginPaint, CreateFontW, CreatePen, CreateRoundRectRgn,
        CreateSolidBrush, DeleteObject, DrawTextW, Ellipse, EndPaint, FillRect, GetMonitorInfoW,
        GetStockObject, InvalidateRect, MonitorFromPoint, RoundRect, SelectObject, SetBkMode,
        SetTextColor, SetWindowRgn, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
        DEFAULT_PITCH, DT_CENTER, DT_END_ELLIPSIS, DT_LEFT, DT_SINGLELINE, DT_VCENTER, FR_NOT_ENUM,
        FR_PRIVATE, FW_NORMAL, FW_SEMIBOLD, HBRUSH, HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        NULL_BRUSH, NULL_PEN, OUT_DEFAULT_PRECIS, PAINTSTRUCT, PS_SOLID, TRANSPARENT,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Input::KeyboardAndMouse::VK_ESCAPE,
        Shell::{
            Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
            NIM_SETVERSION, NIN_SELECT, NOTIFYICONDATAW, NOTIFYICON_VERSION_4,
        },
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorPos, GetMessageW,
            GetSystemMetrics, LoadCursorW, LoadIconW, LoadImageW, PostMessageW, PostQuitMessage,
            RegisterClassW, SetWindowPos, ShowWindow, TranslateMessage, CW_USEDEFAULT, HICON,
            HWND_TOPMOST, IDC_ARROW, IDI_APPLICATION, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE,
            MSG, SM_CXSCREEN, SM_CYSCREEN, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE, WM_CONTEXTMENU,
            WM_DESTROY, WM_KEYDOWN, WM_LBUTTONDBLCLK, WM_LBUTTONUP, WM_PAINT, WM_RBUTTONUP,
            WM_USER, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_OVERLAPPEDWINDOW, WS_POPUP,
        },
    },
};

use crate::{
    config::{local_base, runtime_bind},
    stats::Stats,
    version::{product_name, QORX_VERSION},
};

const TRAY_MESSAGE: u32 = WM_USER + 41;
const UI_REFRESH_MESSAGE: u32 = WM_USER + 42;
const TRAY_ICON_ID: u32 = 1;
const PANEL_WIDTH: i32 = 380;
const PANEL_HEIGHT: i32 = 530;
const PANEL_CORNER_RADIUS: i32 = 18;
const PANEL_MARGIN: i32 = 8;
const PANEL_TRAY_ANCHOR_INSET: i32 = 32;
const PANEL_TASKBAR_GAP: i32 = 8;
const COUNTER_ANIMATION_MS: u64 = 850;
const COUNTER_FRAME_MS: u64 = 33;
const TRAY_CLASS: &str = "QorxVoidNativeTray";
const PANEL_CLASS: &str = "QorxVoidSwitchPanel";

static TRAY_RUNTIME: OnceLock<Mutex<TrayRuntime>> = OnceLock::new();

pub fn run_tray(snapshot: Stats) -> Result<()> {
    load_private_fonts();
    let icon = load_qorx_icon();
    let initial_snapshot = SwitchPanelSnapshot {
        product: product_name().to_string(),
        version: QORX_VERSION.to_string(),
        enabled: false,
        kept_tokens: 0,
        sent_tokens: 0,
        reduction_x: 1.0,
        avoided_usd: snapshot.total_estimated_usd_saved(),
        boot_enabled: false,
    };
    let _ = TRAY_RUNTIME.set(Mutex::new(TrayRuntime {
        tray_hwnd: 0,
        panel_hwnd: 0,
        icon,
        previous_snapshot: initial_snapshot.clone(),
        snapshot: initial_snapshot,
        animation_started_at: None,
        layout: SwitchPanelLayout::default(),
        page: SwitchPanelPage::Switch,
        theme_mode: TrayThemeMode::Light,
    }));

    unsafe {
        let hwnd = create_tray_window(icon)?;
        set_tray_hwnd(hwnd);
        refresh_runtime_snapshot();
        start_refresh_thread();
        run_message_loop();
        remove_tray_icon(hwnd);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayIconAction {
    OpenNativeSwitchPanel,
}

fn tray_icon_click_action() -> TrayIconAction {
    TrayIconAction::OpenNativeSwitchPanel
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwitchPanelOpenStrategy {
    ShowCachedThenRefreshAsync,
}

fn switch_panel_open_strategy() -> SwitchPanelOpenStrategy {
    SwitchPanelOpenStrategy::ShowCachedThenRefreshAsync
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwitchPanelAction {
    ToggleVoid,
    OpenMonitor,
    OpenWorkspace,
    ToggleBoot,
    ToggleTheme,
    HidePanel,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelActionExecution {
    Immediate,
    Background,
}

fn panel_action_execution(action: SwitchPanelAction) -> PanelActionExecution {
    match action {
        SwitchPanelAction::ToggleVoid
        | SwitchPanelAction::OpenMonitor
        | SwitchPanelAction::OpenWorkspace
        | SwitchPanelAction::ToggleBoot => PanelActionExecution::Background,
        SwitchPanelAction::ToggleTheme | SwitchPanelAction::HidePanel | SwitchPanelAction::None => {
            PanelActionExecution::Immediate
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelRepaintStrategy {
    InvalidateOnly,
}

fn panel_repaint_strategy() -> PanelRepaintStrategy {
    PanelRepaintStrategy::InvalidateOnly
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefreshUiStrategy {
    PostMessageToTrayWindow,
}

fn refresh_ui_strategy() -> RefreshUiStrategy {
    RefreshUiStrategy::PostMessageToTrayWindow
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelActivationStrategy {
    TopmostNoActivate,
}

fn panel_activation_strategy() -> PanelActivationStrategy {
    PanelActivationStrategy::TopmostNoActivate
}

fn panel_ex_style() -> u32 {
    match panel_activation_strategy() {
        PanelActivationStrategy::TopmostNoActivate => WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
    }
}

fn panel_show_flags() -> u32 {
    match panel_activation_strategy() {
        PanelActivationStrategy::TopmostNoActivate => SWP_SHOWWINDOW | SWP_NOACTIVATE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwitchPanelPage {
    Switch,
}

#[derive(Debug, Clone, Copy)]
struct HitRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl HitRect {
    fn contains(self, x: i32, y: i32) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}

#[derive(Debug, Clone, Copy)]
struct SwitchPanelLayout {
    switch: HitRect,
    monitor: HitRect,
    workspace: HitRect,
    boot: HitRect,
    theme: HitRect,
    minimize: HitRect,
    close: HitRect,
}

impl Default for SwitchPanelLayout {
    fn default() -> Self {
        Self {
            switch: HitRect {
                left: 132,
                top: 118,
                right: 248,
                bottom: 234,
            },
            monitor: HitRect {
                left: 36,
                top: 424,
                right: 180,
                bottom: 456,
            },
            workspace: HitRect {
                left: 196,
                top: 424,
                right: 344,
                bottom: 456,
            },
            boot: HitRect {
                left: 36,
                top: 464,
                right: 180,
                bottom: 496,
            },
            theme: HitRect {
                left: 196,
                top: 464,
                right: 344,
                bottom: 496,
            },
            minimize: HitRect {
                left: 276,
                top: 18,
                right: 306,
                bottom: 44,
            },
            close: HitRect {
                left: 320,
                top: 18,
                right: 350,
                bottom: 44,
            },
        }
    }
}

impl SwitchPanelLayout {
    fn hit_test(self, x: i32, y: i32) -> SwitchPanelAction {
        if self.switch.contains(x, y) {
            SwitchPanelAction::ToggleVoid
        } else if self.monitor.contains(x, y) {
            SwitchPanelAction::OpenMonitor
        } else if self.workspace.contains(x, y) {
            SwitchPanelAction::OpenWorkspace
        } else if self.boot.contains(x, y) {
            SwitchPanelAction::ToggleBoot
        } else if self.theme.contains(x, y) {
            SwitchPanelAction::ToggleTheme
        } else if self.minimize.contains(x, y) || self.close.contains(x, y) {
            SwitchPanelAction::HidePanel
        } else {
            SwitchPanelAction::None
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SwitchPanelTheme {
    bg: COLORREF,
    paper: COLORREF,
    paper_soft: COLORREF,
    ink: COLORREF,
    muted: COLORREF,
    faint: COLORREF,
    line: COLORREF,
    line_strong: COLORREF,
    good: COLORREF,
    logo: COLORREF,
    ui_font: &'static str,
    display_font: &'static str,
    accent_font: &'static str,
    accent_fallback_font: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayThemeMode {
    Light,
    Dark,
}

impl TrayThemeMode {
    fn toggled(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }

    fn action_label(self) -> &'static str {
        match self {
            Self::Light => "Dark mode",
            Self::Dark => "Light mode",
        }
    }
}

impl SwitchPanelTheme {
    fn monitor(mode: TrayThemeMode) -> Self {
        match mode {
            TrayThemeMode::Light => Self::monitor_light(),
            TrayThemeMode::Dark => Self::monitor_dark(),
        }
    }

    fn monitor_light() -> Self {
        Self {
            bg: rgb(0xf4, 0xf3, 0xee),
            paper: rgb(0xfb, 0xfa, 0xf5),
            paper_soft: rgb(0xee, 0xed, 0xe6),
            ink: rgb(0x19, 0x18, 0x17),
            muted: rgb(0x5a, 0x55, 0x4e),
            faint: rgb(0x8a, 0x84, 0x7a),
            line: rgb(0xd8, 0xd3, 0xc8),
            line_strong: rgb(0xbf, 0xb6, 0xa8),
            good: rgb(0x38, 0x6b, 0x4b),
            logo: rgb(0x05, 0x05, 0x05),
            ui_font: "Geist",
            display_font: "Geist",
            accent_font: "Instrument Serif",
            accent_fallback_font: "Georgia",
        }
    }

    fn monitor_dark() -> Self {
        Self {
            bg: rgb(0x0f, 0x0e, 0x0c),
            paper: rgb(0x17, 0x15, 0x12),
            paper_soft: rgb(0x21, 0x1e, 0x19),
            ink: rgb(0xf7, 0xf3, 0xea),
            muted: rgb(0xb7, 0xaf, 0xa3),
            faint: rgb(0x92, 0x8b, 0x80),
            line: rgb(0x38, 0x33, 0x2b),
            line_strong: rgb(0x5b, 0x53, 0x47),
            good: rgb(0x9f, 0xce, 0xb0),
            logo: rgb(0xf7, 0xf3, 0xea),
            ui_font: "Geist",
            display_font: "Geist",
            accent_font: "Instrument Serif",
            accent_fallback_font: "Georgia",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SwitchPanelTypeScale {
    mark: i32,
    mark_sup: i32,
    switch_mark: i32,
    switch_mark_sup: i32,
    title: i32,
    claim: i32,
    body: i32,
    number: i32,
    small: i32,
}

impl SwitchPanelTypeScale {
    fn compact() -> Self {
        Self {
            mark: -34,
            mark_sup: -12,
            switch_mark: -58,
            switch_mark_sup: -20,
            title: -18,
            claim: -12,
            body: -12,
            number: -17,
            small: -10,
        }
    }
}

struct TrayRuntime {
    tray_hwnd: HWND,
    panel_hwnd: HWND,
    icon: HICON,
    previous_snapshot: SwitchPanelSnapshot,
    snapshot: SwitchPanelSnapshot,
    animation_started_at: Option<Instant>,
    layout: SwitchPanelLayout,
    page: SwitchPanelPage,
    theme_mode: TrayThemeMode,
}

impl TrayRuntime {
    fn set_snapshot(&mut self, snapshot: SwitchPanelSnapshot) -> bool {
        let animate = counters_changed(&self.snapshot, &snapshot);
        if animate {
            self.previous_snapshot = self.animated_snapshot();
            self.animation_started_at = Some(Instant::now());
        }
        self.snapshot = snapshot;
        animate
    }

    fn animated_snapshot(&self) -> SwitchPanelSnapshot {
        let Some(started_at) = self.animation_started_at else {
            return self.snapshot.clone();
        };
        let elapsed = started_at.elapsed();
        if elapsed >= Duration::from_millis(COUNTER_ANIMATION_MS) {
            return self.snapshot.clone();
        }
        let progress = (elapsed.as_secs_f64()
            / Duration::from_millis(COUNTER_ANIMATION_MS).as_secs_f64())
        .clamp(0.0, 1.0);
        interpolate_snapshot(
            &self.previous_snapshot,
            &self.snapshot,
            ease_out_cubic(progress),
        )
    }
}

unsafe fn create_tray_window(icon: HICON) -> Result<HWND> {
    let instance = GetModuleHandleW(ptr::null());
    if instance == 0 {
        anyhow::bail!("failed to get Qorx tray module handle");
    }

    register_window_class(TRAY_CLASS, Some(window_proc), instance);
    register_window_class(PANEL_CLASS, Some(window_proc), instance);

    let hwnd = CreateWindowExW(
        0,
        wide(TRAY_CLASS).as_ptr(),
        wide("Qorx Void Tray").as_ptr(),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        0,
        CW_USEDEFAULT,
        0,
        0,
        0,
        instance,
        ptr::null(),
    );
    if hwnd == 0 {
        anyhow::bail!("failed to create Qorx tray window");
    }

    ShowWindow(hwnd, SW_HIDE);
    update_tray_icon(hwnd, icon, NIM_ADD);
    set_tray_icon_version(hwnd);
    Ok(hwnd)
}

unsafe fn register_window_class(
    class_name: &str,
    proc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
    instance: isize,
) {
    let class = wide(class_name);
    let mut wnd = mem::zeroed::<WNDCLASSW>();
    wnd.lpfnWndProc = proc;
    wnd.hInstance = instance;
    wnd.lpszClassName = class.as_ptr();
    wnd.hCursor = LoadCursorW(0, IDC_ARROW);
    RegisterClassW(&wnd);
}

unsafe fn run_message_loop() {
    let mut msg = mem::zeroed::<MSG>();
    while GetMessageW(&mut msg, 0, 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == UI_REFRESH_MESSAGE {
        apply_runtime_ui_refresh();
        return 0;
    }

    if msg == TRAY_MESSAGE && tray_icon_event(l_param) {
        match tray_icon_click_action() {
            TrayIconAction::OpenNativeSwitchPanel => match switch_panel_open_strategy() {
                SwitchPanelOpenStrategy::ShowCachedThenRefreshAsync => show_switch_panel(hwnd),
            },
        }
        return 0;
    }

    if is_panel_window(hwnd) {
        match msg {
            WM_PAINT => {
                draw_switch_panel(hwnd);
                return 0;
            }
            WM_LBUTTONUP => {
                handle_panel_click(hwnd, point_from_lparam(l_param));
                return 0;
            }
            WM_KEYDOWN => {
                if w_param as u16 == VK_ESCAPE {
                    ShowWindow(hwnd, SW_HIDE);
                    return 0;
                }
            }
            WM_DESTROY => {
                set_panel_hwnd(0);
                return 0;
            }
            _ => {}
        }
    }

    match msg {
        WM_DESTROY => {
            remove_tray_icon(hwnd);
            PostQuitMessage(0);
            return 0;
        }
        _ => {}
    }

    DefWindowProcW(hwnd, msg, w_param, l_param)
}

fn tray_icon_event(l_param: LPARAM) -> bool {
    let event = (l_param as u32) & 0xffff;
    matches!(
        event,
        WM_LBUTTONUP | WM_RBUTTONUP | WM_LBUTTONDBLCLK | WM_CONTEXTMENU | NIN_SELECT
    )
}

unsafe fn show_switch_panel(owner: HWND) {
    let mut point = POINT { x: 0, y: 0 };
    if GetCursorPos(&mut point) == 0 {
        return;
    }

    let hwnd = existing_or_create_panel(owner);
    if hwnd == 0 {
        return;
    }

    let (x, y) = panel_position(point);
    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        x,
        y,
        PANEL_WIDTH,
        PANEL_HEIGHT,
        panel_show_flags(),
    );
    request_panel_repaint(hwnd);
}

fn start_refresh_thread() {
    thread::spawn(|| loop {
        refresh_runtime_snapshot_data();
        request_runtime_ui_refresh();
        thread::sleep(Duration::from_millis(2_500));
    });
}

unsafe fn existing_or_create_panel(owner: HWND) -> HWND {
    if let Some(panel) = with_runtime(|runtime| runtime.panel_hwnd) {
        if panel != 0 {
            return panel;
        }
    }

    let instance = GetModuleHandleW(ptr::null());
    let hwnd = CreateWindowExW(
        panel_ex_style(),
        wide(PANEL_CLASS).as_ptr(),
        wide("Qorx Void").as_ptr(),
        WS_POPUP,
        0,
        0,
        PANEL_WIDTH,
        PANEL_HEIGHT,
        owner,
        0,
        instance,
        ptr::null(),
    );
    if hwnd != 0 {
        apply_panel_rounding(hwnd);
    }
    set_panel_hwnd(hwnd);
    hwnd
}

unsafe fn apply_panel_rounding(hwnd: HWND) {
    let region = CreateRoundRectRgn(
        0,
        0,
        PANEL_WIDTH + 1,
        PANEL_HEIGHT + 1,
        PANEL_CORNER_RADIUS,
        PANEL_CORNER_RADIUS,
    );
    if region != 0 {
        SetWindowRgn(hwnd, region, 1);
    }
}

fn panel_position(point: POINT) -> (i32, i32) {
    unsafe {
        let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
        if monitor != 0 {
            let mut info = mem::zeroed::<MONITORINFO>();
            info.cbSize = mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(monitor, &mut info) != 0 {
                return panel_position_in_work_area(point, info.rcMonitor, info.rcWork);
            }
        }
        let screen = rect(
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
        );
        panel_position_in_work_area(point, screen, screen)
    }
}

fn panel_position_in_work_area(point: POINT, monitor: RECT, work: RECT) -> (i32, i32) {
    let min_x = work.left + PANEL_MARGIN;
    let max_x = (work.right - PANEL_WIDTH - PANEL_MARGIN).max(min_x);
    let min_y = work.top + PANEL_MARGIN;
    let max_y = (work.bottom - PANEL_HEIGHT - PANEL_MARGIN).max(min_y);

    let mut x = point.x - PANEL_WIDTH + PANEL_TRAY_ANCHOR_INSET;
    let mut y = point.y - PANEL_HEIGHT - PANEL_TASKBAR_GAP;

    if work.bottom < monitor.bottom {
        y = work.bottom - PANEL_HEIGHT - PANEL_TASKBAR_GAP;
    } else if work.top > monitor.top {
        y = work.top + PANEL_TASKBAR_GAP;
    } else if work.right < monitor.right {
        x = work.right - PANEL_WIDTH - PANEL_TASKBAR_GAP;
        y = point.y - PANEL_HEIGHT + PANEL_TRAY_ANCHOR_INSET;
    } else if work.left > monitor.left {
        x = work.left + PANEL_TASKBAR_GAP;
        y = point.y - PANEL_HEIGHT + PANEL_TRAY_ANCHOR_INSET;
    }

    (x.clamp(min_x, max_x), y.clamp(min_y, max_y))
}

unsafe fn handle_panel_click(hwnd: HWND, point: (i32, i32)) {
    let action = with_runtime(|runtime| runtime.layout.hit_test(point.0, point.1))
        .unwrap_or(SwitchPanelAction::None);
    if panel_action_execution(action) == PanelActionExecution::Background {
        start_panel_background_action(hwnd, action);
        return;
    }

    match action {
        SwitchPanelAction::ToggleTheme => {
            let _ = with_runtime_mut(|runtime| runtime.theme_mode = runtime.theme_mode.toggled());
            request_panel_repaint(hwnd);
        }
        SwitchPanelAction::HidePanel => {
            ShowWindow(hwnd, SW_HIDE);
        }
        SwitchPanelAction::ToggleVoid
        | SwitchPanelAction::OpenMonitor
        | SwitchPanelAction::OpenWorkspace
        | SwitchPanelAction::ToggleBoot
        | SwitchPanelAction::None => {}
    }
}

unsafe fn start_panel_background_action(hwnd: HWND, action: SwitchPanelAction) {
    match action {
        SwitchPanelAction::ToggleVoid => {
            let _ =
                with_runtime_mut(|runtime| runtime.snapshot.enabled = !runtime.snapshot.enabled);
            request_panel_repaint(hwnd);
            thread::spawn(|| {
                toggle_qorx_switch();
                thread::sleep(Duration::from_millis(300));
                refresh_runtime_snapshot_data();
                request_runtime_ui_refresh();
            });
        }
        SwitchPanelAction::OpenMonitor => {
            thread::spawn(|| {
                let _ = open::that(format!("{}/monitor", local_base()));
            });
        }
        SwitchPanelAction::OpenWorkspace => {
            thread::spawn(|| {
                let _ = open_workspace();
            });
        }
        SwitchPanelAction::ToggleBoot => {
            let next_enabled = with_runtime_mut(|runtime| {
                runtime.snapshot.boot_enabled = !runtime.snapshot.boot_enabled;
                runtime.snapshot.boot_enabled
            })
            .unwrap_or(true);
            request_panel_repaint(hwnd);
            thread::spawn(move || {
                set_start_on_boot(next_enabled);
                thread::sleep(Duration::from_millis(300));
                refresh_runtime_snapshot_data();
                request_runtime_ui_refresh();
            });
        }
        SwitchPanelAction::ToggleTheme | SwitchPanelAction::HidePanel | SwitchPanelAction::None => {
        }
    }
}

unsafe fn request_panel_repaint(hwnd: HWND) {
    match panel_repaint_strategy() {
        PanelRepaintStrategy::InvalidateOnly => {
            InvalidateRect(hwnd, ptr::null(), 1);
        }
    }
}

unsafe fn draw_switch_panel(hwnd: HWND) {
    let mut paint = mem::zeroed::<PAINTSTRUCT>();
    let hdc = BeginPaint(hwnd, &mut paint);
    if hdc == 0 {
        return;
    }

    let (snapshot, page, theme_mode) = with_runtime(|runtime| {
        (
            runtime.animated_snapshot(),
            runtime.page,
            runtime.theme_mode,
        )
    })
    .unwrap_or_else(|| {
        (
            SwitchPanelSnapshot {
                product: product_name().to_string(),
                version: QORX_VERSION.to_string(),
                enabled: false,
                kept_tokens: 0,
                sent_tokens: 0,
                reduction_x: 1.0,
                avoided_usd: 0.0,
                boot_enabled: false,
            },
            SwitchPanelPage::Switch,
            TrayThemeMode::Light,
        )
    });
    let labels = switch_panel_labels(snapshot.clone());
    let theme = SwitchPanelTheme::monitor(theme_mode);

    fill_rect(hdc, rect(0, 0, PANEL_WIDTH, PANEL_HEIGHT), theme.paper);

    let scale = SwitchPanelTypeScale::compact();
    let accent_family = accent_font_family(theme);
    let mark_font = create_font(scale.mark, FW_NORMAL as i32, accent_family, true);
    let mark_sup_font = create_font(scale.mark_sup, FW_NORMAL as i32, accent_family, true);
    let switch_mark_font = create_font(scale.switch_mark, FW_NORMAL as i32, accent_family, true);
    let switch_mark_sup_font =
        create_font(scale.switch_mark_sup, FW_NORMAL as i32, accent_family, true);
    let title_font = create_font(scale.title, FW_SEMIBOLD as i32, theme.display_font, false);
    let claim_font = create_font(scale.claim, FW_NORMAL as i32, accent_family, false);
    let body_font = create_font(scale.body, FW_NORMAL as i32, theme.ui_font, false);
    let number_font = create_font(scale.number, FW_NORMAL as i32, theme.display_font, false);
    let small_font = create_font(scale.small, FW_SEMIBOLD as i32, theme.ui_font, false);

    draw_qorx_mark(
        hdc,
        rect(32, 25, 66, 61),
        mark_font,
        mark_sup_font,
        theme.logo,
    );
    draw_text(
        hdc,
        &labels.title,
        rect(68, 29, 228, 53),
        title_font,
        theme.ink,
        DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS,
    );
    draw_text(
        hdc,
        &format!("v{}", labels.version),
        rect(69, 54, 230, 72),
        small_font,
        theme.muted,
        DT_LEFT | DT_SINGLELINE,
    );
    draw_text(
        hdc,
        "-",
        rect(276, 18, 306, 44),
        body_font,
        theme.faint,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER,
    );
    draw_text(
        hdc,
        "x",
        rect(320, 18, 350, 44),
        body_font,
        theme.faint,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER,
    );
    draw_text(
        hdc,
        "Local context, kept local.",
        rect(30, 82, PANEL_WIDTH - 30, 106),
        claim_font,
        theme.ink,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER,
    );

    match page {
        SwitchPanelPage::Switch => draw_switch_page(
            hdc,
            theme,
            &snapshot,
            &labels,
            switch_mark_font,
            switch_mark_sup_font,
            body_font,
            number_font,
            small_font,
            theme_mode,
        ),
    }

    DeleteObject(mark_font as HGDIOBJ);
    DeleteObject(mark_sup_font as HGDIOBJ);
    DeleteObject(switch_mark_font as HGDIOBJ);
    DeleteObject(switch_mark_sup_font as HGDIOBJ);
    DeleteObject(title_font as HGDIOBJ);
    DeleteObject(claim_font as HGDIOBJ);
    DeleteObject(body_font as HGDIOBJ);
    DeleteObject(number_font as HGDIOBJ);
    DeleteObject(small_font as HGDIOBJ);
    EndPaint(hwnd, &paint);
}

unsafe fn draw_switch_page(
    hdc: isize,
    theme: SwitchPanelTheme,
    snapshot: &SwitchPanelSnapshot,
    labels: &SwitchPanelLabels,
    mark_font: isize,
    sup_font: isize,
    body_font: isize,
    number_font: isize,
    small_font: isize,
    theme_mode: TrayThemeMode,
) {
    draw_switch(hdc, theme, mark_font, sup_font, snapshot.enabled);
    draw_text(
        hdc,
        if snapshot.enabled {
            "Qorx is on"
        } else {
            "Qorx is off"
        },
        rect(40, 244, 340, 266),
        body_font,
        theme.ink,
        DT_CENTER | DT_SINGLELINE,
    );
    let status_text = if snapshot.enabled {
        "MCP + hooks are connected."
    } else {
        "Click to turn on MCP + hooks."
    };
    fill_round_rect(hdc, rect(66, 274, 314, 304), theme.paper_soft, 15);
    fill_ellipse(
        hdc,
        rect(86, 285, 95, 294),
        if snapshot.enabled {
            theme.good
        } else {
            theme.faint
        },
    );
    draw_text(
        hdc,
        status_text,
        rect(108, 278, 294, 301),
        small_font,
        theme.muted,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
    );

    draw_metric(
        hdc,
        theme,
        body_font,
        number_font,
        36,
        314,
        "Kept here",
        &strip_metric_value(&labels.kept),
    );
    draw_metric(
        hdc,
        theme,
        body_font,
        number_font,
        198,
        314,
        "Sent to AI",
        &strip_metric_value(&labels.sent),
    );
    draw_metric(
        hdc,
        theme,
        body_font,
        number_font,
        36,
        370,
        "Reduction",
        &strip_metric_value(&labels.reduction),
    );
    draw_metric(
        hdc,
        theme,
        body_font,
        number_font,
        198,
        370,
        "Avoided cost",
        &strip_metric_value(&labels.avoided),
    );

    draw_button(
        hdc,
        theme,
        rect(36, 424, 180, 456),
        "Preferences",
        body_font,
    );
    draw_button(hdc, theme, rect(196, 424, 344, 456), "Workspace", body_font);
    draw_button(
        hdc,
        theme,
        rect(36, 464, 180, 496),
        if snapshot.boot_enabled {
            "Boot on"
        } else {
            "Start on boot"
        },
        body_font,
    );
    draw_button(
        hdc,
        theme,
        rect(196, 464, 344, 496),
        theme_mode.action_label(),
        body_font,
    );
}

unsafe fn draw_switch(
    hdc: isize,
    theme: SwitchPanelTheme,
    mark_font: isize,
    sup_font: isize,
    enabled: bool,
) {
    let orb = rect(132, 118, 248, 234);
    let fill = if enabled { theme.ink } else { theme.paper };
    let text = if enabled { theme.bg } else { theme.logo };
    fill_ellipse(hdc, orb, fill);
    if !enabled {
        stroke_round_rect(hdc, orb, theme.line, 148);
    }
    draw_qorx_mark(hdc, rect(157, 146, 224, 203), mark_font, sup_font, text);
}

unsafe fn draw_qorx_mark(
    hdc: isize,
    bounds: RECT,
    mark_font: isize,
    sup_font: isize,
    color: COLORREF,
) {
    let height = bounds.bottom - bounds.top;
    let q_width = ((bounds.right - bounds.left) as f32 * 0.72) as i32;
    draw_text(
        hdc,
        "Q",
        rect(
            bounds.left,
            bounds.top,
            bounds.left + q_width,
            bounds.bottom,
        ),
        mark_font,
        color,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER,
    );
    draw_text(
        hdc,
        "x",
        rect(
            bounds.left + q_width - 2,
            bounds.top + (height / 7),
            bounds.right,
            bounds.top + (height / 2),
        ),
        sup_font,
        color,
        DT_LEFT | DT_SINGLELINE | DT_VCENTER,
    );
}

unsafe fn draw_metric(
    hdc: isize,
    theme: SwitchPanelTheme,
    label_font: isize,
    value_font: isize,
    x: i32,
    y: i32,
    label: &str,
    value: &str,
) {
    fill_round_rect(hdc, rect(x, y, x + 146, y + 46), theme.paper_soft, 10);
    stroke_round_rect(hdc, rect(x, y, x + 146, y + 46), theme.line, 10);
    draw_text(
        hdc,
        label,
        rect(x + 10, y + 8, x + 136, y + 22),
        label_font,
        theme.muted,
        DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS,
    );
    draw_text(
        hdc,
        value,
        rect(x + 10, y + 23, x + 136, y + 44),
        value_font,
        theme.ink,
        DT_LEFT | DT_SINGLELINE | DT_END_ELLIPSIS,
    );
}

unsafe fn draw_button(hdc: isize, theme: SwitchPanelTheme, bounds: RECT, label: &str, font: isize) {
    fill_round_rect(hdc, bounds, theme.paper, 8);
    stroke_round_rect(hdc, bounds, theme.line_strong, 8);
    draw_text(
        hdc,
        label,
        bounds,
        font,
        theme.ink,
        DT_CENTER | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
    );
}

unsafe fn draw_text(
    hdc: isize,
    text: &str,
    mut bounds: RECT,
    font: isize,
    color: COLORREF,
    flags: u32,
) {
    let old_font = SelectObject(hdc, font);
    SetBkMode(hdc, TRANSPARENT as i32);
    SetTextColor(hdc, color);
    let wide = wide(text);
    DrawTextW(hdc, wide.as_ptr(), -1, &mut bounds, flags);
    SelectObject(hdc, old_font);
}

unsafe fn fill_rect(hdc: isize, bounds: RECT, color: COLORREF) {
    let brush = CreateSolidBrush(color);
    FillRect(hdc, &bounds, brush);
    DeleteObject(brush as HGDIOBJ);
}

unsafe fn fill_round_rect(hdc: isize, bounds: RECT, color: COLORREF, radius: i32) {
    let brush: HBRUSH = CreateSolidBrush(color);
    let old_brush = SelectObject(hdc, brush as HGDIOBJ);
    let old_pen = SelectObject(hdc, GetStockObject(NULL_PEN));
    RoundRect(
        hdc,
        bounds.left,
        bounds.top,
        bounds.right,
        bounds.bottom,
        radius,
        radius,
    );
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
    DeleteObject(brush as HGDIOBJ);
}

unsafe fn fill_ellipse(hdc: isize, bounds: RECT, color: COLORREF) {
    let brush: HBRUSH = CreateSolidBrush(color);
    let old_brush = SelectObject(hdc, brush as HGDIOBJ);
    let old_pen = SelectObject(hdc, GetStockObject(NULL_PEN));
    Ellipse(hdc, bounds.left, bounds.top, bounds.right, bounds.bottom);
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);
    DeleteObject(brush as HGDIOBJ);
}

unsafe fn stroke_round_rect(hdc: isize, bounds: RECT, color: COLORREF, radius: i32) {
    let pen = CreatePen(PS_SOLID, 1, color);
    let old_pen = SelectObject(hdc, pen as HGDIOBJ);
    let old_brush = SelectObject(hdc, GetStockObject(NULL_BRUSH));
    RoundRect(
        hdc,
        bounds.left,
        bounds.top,
        bounds.right,
        bounds.bottom,
        radius,
        radius,
    );
    SelectObject(hdc, old_brush);
    SelectObject(hdc, old_pen);
    DeleteObject(pen as HGDIOBJ);
}

unsafe fn create_font(height: i32, weight: i32, family: &str, italic: bool) -> isize {
    CreateFontW(
        height,
        0,
        0,
        0,
        weight,
        u32::from(italic),
        0,
        0,
        DEFAULT_CHARSET as u32,
        OUT_DEFAULT_PRECIS as u32,
        CLIP_DEFAULT_PRECIS as u32,
        CLEARTYPE_QUALITY as u32,
        DEFAULT_PITCH as u32,
        wide(family).as_ptr(),
    )
}

fn accent_font_family(theme: SwitchPanelTheme) -> &'static str {
    if theme.accent_font.is_empty() {
        theme.accent_fallback_font
    } else {
        theme.accent_font
    }
}

fn load_private_fonts() {
    for path in private_font_paths() {
        let wide_path = wide(&path.display().to_string());
        unsafe {
            AddFontResourceExW(wide_path.as_ptr(), FR_PRIVATE | FR_NOT_ENUM, ptr::null());
        }
    }
}

fn private_font_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("fonts"));
            dirs.push(
                parent
                    .parent()
                    .and_then(|target| target.parent())
                    .map(|repo| repo.join("fonts"))
                    .unwrap_or_else(|| parent.join("fonts")),
            );
        }
    }
    for dir in dirs {
        for name in private_font_file_names() {
            let path = dir.join(name);
            if path.exists() {
                paths.push(path);
            }
        }
    }
    paths
}

fn private_font_file_names() -> [&'static str; 4] {
    [
        "Geist[wght].ttf",
        "GeistMono[wght].ttf",
        "InstrumentSerif-Regular.ttf",
        "InstrumentSerif-Italic.ttf",
    ]
}

fn strip_metric_value(label: &str) -> String {
    label
        .split_once(": ")
        .map(|(_, value)| value.to_string())
        .unwrap_or_else(|| label.to_string())
}

fn refresh_runtime_snapshot() {
    if refresh_runtime_snapshot_data() {
        unsafe { apply_runtime_ui_refresh() };
    }
}

fn refresh_runtime_snapshot_data() -> bool {
    let Some(stats) = fetch_tray_stats() else {
        return false;
    };
    let integrations = fetch_tray_integrations();
    let snapshot = switch_panel_snapshot(stats, integrations.as_ref());
    let mut animate = false;
    with_runtime_mut(|runtime| {
        animate = runtime.set_snapshot(snapshot);
    })
    .is_some()
    .then(|| {
        if animate {
            start_counter_animation_frames();
        }
    })
    .is_some()
}

fn start_counter_animation_frames() {
    let Some(hwnd) = with_runtime(|runtime| runtime.tray_hwnd) else {
        return;
    };
    if hwnd == 0 {
        return;
    }
    thread::spawn(move || {
        let frames = (COUNTER_ANIMATION_MS / COUNTER_FRAME_MS).max(1);
        for _ in 0..=frames {
            unsafe {
                PostMessageW(hwnd, UI_REFRESH_MESSAGE, 0, 0);
            }
            thread::sleep(Duration::from_millis(COUNTER_FRAME_MS));
        }
    });
}

fn request_runtime_ui_refresh() {
    match refresh_ui_strategy() {
        RefreshUiStrategy::PostMessageToTrayWindow => {
            if let Some(hwnd) = with_runtime(|runtime| runtime.tray_hwnd) {
                if hwnd != 0 {
                    unsafe {
                        PostMessageW(hwnd, UI_REFRESH_MESSAGE, 0, 0);
                    }
                }
            }
        }
    }
}

unsafe fn apply_runtime_ui_refresh() {
    if let Some((hwnd, icon)) = with_runtime(|runtime| (runtime.tray_hwnd, runtime.icon)) {
        if hwnd != 0 {
            update_tray_icon(hwnd, icon, NIM_MODIFY);
        }
    }
    if let Some(hwnd) = with_runtime(|runtime| runtime.panel_hwnd) {
        if hwnd != 0 {
            request_panel_repaint(hwnd);
        }
    }
}

unsafe fn update_tray_icon(hwnd: HWND, icon: HICON, message: u32) {
    let tooltip = with_runtime(|runtime| {
        let labels = switch_panel_labels(runtime.snapshot.clone());
        format!(
            "{} v{} | {} | {} | {}",
            labels.title, labels.version, labels.state, labels.toggle, labels.avoided
        )
    })
    .unwrap_or_else(|| format!("{} v{}", product_name(), QORX_VERSION));
    let wide_tooltip = wide(&tooltip);
    let mut data = mem::zeroed::<NOTIFYICONDATAW>();
    data.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_ICON_ID;
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = TRAY_MESSAGE;
    data.hIcon = icon;
    let copy_len = wide_tooltip.len().min(data.szTip.len() - 1);
    data.szTip[..copy_len].copy_from_slice(&wide_tooltip[..copy_len]);
    Shell_NotifyIconW(message, &data);
}

unsafe fn set_tray_icon_version(hwnd: HWND) {
    let mut data = mem::zeroed::<NOTIFYICONDATAW>();
    data.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_ICON_ID;
    data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
    Shell_NotifyIconW(NIM_SETVERSION, &data);
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut data = mem::zeroed::<NOTIFYICONDATAW>();
    data.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_ICON_ID;
    Shell_NotifyIconW(NIM_DELETE, &data);
}

fn set_tray_hwnd(hwnd: HWND) {
    let _ = with_runtime_mut(|runtime| runtime.tray_hwnd = hwnd);
}

fn set_panel_hwnd(hwnd: HWND) {
    let _ = with_runtime_mut(|runtime| runtime.panel_hwnd = hwnd);
}

fn is_panel_window(hwnd: HWND) -> bool {
    with_runtime(|runtime| runtime.panel_hwnd == hwnd).unwrap_or(false)
}

fn with_runtime<T>(f: impl FnOnce(&TrayRuntime) -> T) -> Option<T> {
    TRAY_RUNTIME
        .get()
        .and_then(|state| state.lock().ok().map(|runtime| f(&runtime)))
}

fn with_runtime_mut<T>(f: impl FnOnce(&mut TrayRuntime) -> T) -> Option<T> {
    TRAY_RUNTIME
        .get()
        .and_then(|state| state.lock().ok().map(|mut runtime| f(&mut runtime)))
}

fn point_from_lparam(l_param: LPARAM) -> (i32, i32) {
    let raw = l_param as i32;
    let x = (raw & 0xffff) as i16 as i32;
    let y = ((raw >> 16) & 0xffff) as i16 as i32;
    (x, y)
}

fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    red as u32 | ((green as u32) << 8) | ((blue as u32) << 16)
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[derive(Debug, Deserialize)]
struct TrayStats {
    product: Option<String>,
    version: Option<String>,
    context_omitted_tokens: Option<u64>,
    context_sent_tokens: Option<u64>,
    compressed_prompt_tokens: Option<u64>,
    context_reduction_x: Option<f64>,
    total_estimated_usd_saved: Option<f64>,
    session: Option<TraySessionStats>,
    live: Option<TrayLiveStats>,
}

#[derive(Debug, Deserialize)]
struct TraySessionStats {
    saved_prompt_tokens: Option<u64>,
    context_omitted_tokens: Option<u64>,
    context_sent_tokens: Option<u64>,
    compressed_prompt_tokens: Option<u64>,
    context_reduction_x: Option<f64>,
    total_estimated_usd_saved: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TrayLiveStats {
    last_reduction_x: Option<f64>,
}

impl Default for TraySessionStats {
    fn default() -> Self {
        Self {
            saved_prompt_tokens: Some(0),
            context_omitted_tokens: Some(0),
            context_sent_tokens: Some(0),
            compressed_prompt_tokens: Some(0),
            context_reduction_x: Some(1.0),
            total_estimated_usd_saved: Some(0.0),
        }
    }
}

impl Default for TrayLiveStats {
    fn default() -> Self {
        Self {
            last_reduction_x: Some(1.0),
        }
    }
}

fn fetch_tray_stats() -> Option<TrayStats> {
    fetch_tray_stats_with_timeout(Duration::from_secs(2))
}

fn fetch_tray_stats_with_timeout(timeout: Duration) -> Option<TrayStats> {
    fetch_runtime_body("/stats", timeout).and_then(|body| serde_json::from_str(&body).ok())
}

fn fetch_runtime_body(path: &str, timeout: Duration) -> Option<String> {
    let addr = runtime_bind().parse().ok()?;
    let mut stream = TcpStream::connect_timeout(&addr, timeout).ok()?;
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let request = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).ok()?;
    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;
    let body = response.split("\r\n\r\n").nth(1)?;
    Some(body.to_string())
}

#[derive(Debug, Deserialize)]
struct TrayIntegrationReport {
    settings: TrayIntegrationSettings,
    autostart: Option<TrayIntegrationStatus>,
}

#[derive(Debug, Default, Deserialize)]
struct TrayIntegrationSettings {
    automcp_enabled: bool,
    autohook_enabled: bool,
}

#[derive(Debug, Default, Deserialize)]
struct TrayIntegrationStatus {
    active: bool,
}

fn fetch_tray_integrations() -> Option<TrayIntegrationReport> {
    fetch_runtime_body("/integrations", Duration::from_secs(2))
        .and_then(|body| serde_json::from_str(&body).ok())
}

#[derive(Debug, Clone)]
struct SwitchPanelSnapshot {
    product: String,
    version: String,
    enabled: bool,
    kept_tokens: u64,
    sent_tokens: u64,
    reduction_x: f64,
    avoided_usd: f64,
    boot_enabled: bool,
}

#[derive(Debug, Clone)]
struct SwitchPanelLabels {
    title: String,
    version: String,
    state: String,
    toggle: String,
    kept: String,
    sent: String,
    reduction: String,
    avoided: String,
}

fn switch_panel_snapshot(
    stats: TrayStats,
    integrations: Option<&TrayIntegrationReport>,
) -> SwitchPanelSnapshot {
    let session = stats.session.unwrap_or_default();
    let live = stats.live.unwrap_or_default();
    let context_kept = session
        .context_omitted_tokens
        .or(stats.context_omitted_tokens)
        .unwrap_or(0);
    let proxy_kept = session.saved_prompt_tokens.unwrap_or(0);
    let kept_tokens = if context_kept > 0 {
        context_kept
    } else {
        proxy_kept
    };
    let context_sent = session
        .context_sent_tokens
        .or(stats.context_sent_tokens)
        .unwrap_or(0);
    let proxy_sent = session
        .compressed_prompt_tokens
        .or(stats.compressed_prompt_tokens)
        .unwrap_or(0);
    let sent_tokens = if context_sent > 0 {
        context_sent
    } else {
        proxy_sent
    };
    let inferred_reduction = if kept_tokens > 0 && sent_tokens > 0 {
        (kept_tokens + sent_tokens) as f64 / sent_tokens as f64
    } else {
        1.0
    };
    let reduction_x = session
        .context_reduction_x
        .or(stats.context_reduction_x)
        .or(live.last_reduction_x)
        .unwrap_or(inferred_reduction)
        .max(inferred_reduction);
    let enabled = integrations
        .map(|report| report.settings.automcp_enabled && report.settings.autohook_enabled)
        .unwrap_or(false);
    let boot_enabled = integrations
        .and_then(|report| report.autostart.as_ref())
        .map(|status| status.active)
        .unwrap_or(false);

    SwitchPanelSnapshot {
        product: stats.product.unwrap_or_else(|| product_name().to_string()),
        version: stats.version.unwrap_or_else(|| QORX_VERSION.to_string()),
        enabled,
        kept_tokens,
        sent_tokens,
        reduction_x,
        avoided_usd: session
            .total_estimated_usd_saved
            .or(stats.total_estimated_usd_saved)
            .unwrap_or(0.0),
        boot_enabled,
    }
}

fn switch_panel_labels(snapshot: SwitchPanelSnapshot) -> SwitchPanelLabels {
    SwitchPanelLabels {
        title: snapshot.product,
        version: snapshot.version,
        state: if snapshot.enabled {
            "Status: On".to_string()
        } else {
            "Status: Off".to_string()
        },
        toggle: if snapshot.enabled {
            "Turn Qorx Off".to_string()
        } else {
            "Turn Qorx On".to_string()
        },
        kept: format!("Kept here: {}", token_label(snapshot.kept_tokens)),
        sent: format!("Sent to AI: {}", token_label(snapshot.sent_tokens)),
        reduction: if snapshot.reduction_x > 1.0 {
            format!("Reduction: {:.0}x", snapshot.reduction_x)
        } else {
            "Reduction: waiting".to_string()
        },
        avoided: format!("Avoided input cost: {}", money_label(snapshot.avoided_usd)),
    }
}

fn counters_changed(current: &SwitchPanelSnapshot, next: &SwitchPanelSnapshot) -> bool {
    current.kept_tokens != next.kept_tokens
        || current.sent_tokens != next.sent_tokens
        || (current.reduction_x - next.reduction_x).abs() > 0.01
        || (current.avoided_usd - next.avoided_usd).abs() > 0.0001
}

fn ease_out_cubic(progress: f64) -> f64 {
    1.0 - (1.0 - progress).powi(3)
}

fn interpolate_snapshot(
    from: &SwitchPanelSnapshot,
    to: &SwitchPanelSnapshot,
    progress: f64,
) -> SwitchPanelSnapshot {
    SwitchPanelSnapshot {
        product: to.product.clone(),
        version: to.version.clone(),
        enabled: to.enabled,
        kept_tokens: lerp_u64(from.kept_tokens, to.kept_tokens, progress),
        sent_tokens: lerp_u64(from.sent_tokens, to.sent_tokens, progress),
        reduction_x: lerp_f64(from.reduction_x, to.reduction_x, progress),
        avoided_usd: lerp_f64(from.avoided_usd, to.avoided_usd, progress),
        boot_enabled: to.boot_enabled,
    }
}

fn lerp_u64(from: u64, to: u64, progress: f64) -> u64 {
    let value = lerp_f64(from as f64, to as f64, progress).round();
    value.max(0.0) as u64
}

fn lerp_f64(from: f64, to: f64, progress: f64) -> f64 {
    from + (to - from) * progress.clamp(0.0, 1.0)
}

fn toggle_qorx_switch() {
    let enabled = fetch_tray_integrations()
        .map(|report| report.settings.automcp_enabled && report.settings.autohook_enabled)
        .unwrap_or(false);
    if enabled {
        run_qorx_command(&[
            "integrate",
            "settings",
            "--automcp",
            "false",
            "--autohook",
            "false",
        ]);
    } else {
        run_qorx_command(&["integrate", "activate", "--platform", "all"]);
    }
}

fn set_start_on_boot(enabled: bool) {
    if enabled {
        run_qorx_command(&["startup", "enable"]);
    } else {
        run_qorx_command(&["startup", "disable"]);
    }
}

fn open_workspace() -> std::io::Result<()> {
    for path in workspace_candidates() {
        if path.exists() {
            return open::that(path);
        }
    }
    open::that(format!("{}/monitor", local_base()))
}

fn workspace_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = std::env::var_os("QORX_WORKSPACE") {
        candidates.push(PathBuf::from(path));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(release) = exe.parent() {
            if let Some(target) = release.parent() {
                if let Some(repo) = target.parent() {
                    if let Some(local_pro) = repo.parent() {
                        if let Some(workspace) = local_pro.parent() {
                            candidates.push(workspace.to_path_buf());
                        }
                    }
                    candidates.push(repo.to_path_buf());
                }
            }
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir);
    }

    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));

    let mut unique = Vec::new();
    for candidate in candidates {
        if !unique.iter().any(|path| path == &candidate) {
            unique.push(candidate);
        }
    }
    unique
}

fn token_label(tokens: u64) -> String {
    if tokens >= 1_000_000_000 {
        format!("{:.2}B", tokens as f64 / 1_000_000_000.0)
    } else if tokens >= 1_000_000 {
        format!("{:.2}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

fn money_label(value: f64) -> String {
    if value.abs() >= 10.0 {
        format!("${value:.2}")
    } else {
        format!("${value:.4}")
    }
}

fn run_qorx_command(args: &[&str]) {
    if let Ok(exe) = std::env::current_exe() {
        let mut command = Command::new(exe);
        command
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        configure_hidden_process(&mut command);
        let _ = command.spawn();
    }
}

fn load_qorx_icon() -> HICON {
    let exe = std::env::current_exe().ok();
    for icon_path in qorx_icon_candidates(exe.as_deref()) {
        if !icon_path.exists() {
            continue;
        }
        let wide = wide(&icon_path.display().to_string());
        let loaded = unsafe {
            LoadImageW(
                0,
                wide.as_ptr(),
                IMAGE_ICON,
                0,
                0,
                LR_LOADFROMFILE | LR_DEFAULTSIZE,
            ) as HICON
        };
        if loaded != 0 {
            return loaded;
        }
    }

    unsafe { LoadIconW(0, IDI_APPLICATION) as HICON }
}

fn qorx_icon_candidates(exe: Option<&Path>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = std::env::var_os("QORX_ICON_PATH") {
        candidates.push(PathBuf::from(path));
    }

    if let Some(exe) = exe {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("icon").join("qorx-ico.ico"));
            if let Some(cargo_or_target) = parent.parent() {
                candidates.push(cargo_or_target.join("icon").join("qorx-ico.ico"));
                if let Some(repo_or_home) = cargo_or_target.parent() {
                    candidates.push(repo_or_home.join("icon").join("qorx-ico.ico"));
                }
            }
        }
    }

    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("icon")
            .join("qorx-ico.ico"),
    );
    candidates.push(PathBuf::from(r"C:\Qorx\icon\qorx-ico.ico"));

    let mut unique = Vec::new();
    for candidate in candidates {
        if !unique.iter().any(|path| path == &candidate) {
            unique.push(candidate);
        }
    }
    unique
}

fn configure_hidden_process(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW);
}

#[cfg(test)]
mod tests {
    #[test]
    fn formats_compact_token_labels() {
        assert_eq!(super::token_label(999), "999");
        assert_eq!(super::token_label(1_200), "1.2K");
        assert_eq!(super::token_label(2_300_000), "2.30M");
    }

    #[test]
    fn formats_compact_money_labels() {
        assert_eq!(super::money_label(0.12345), "$0.1235");
        assert_eq!(super::money_label(12.345), "$12.35");
    }

    #[test]
    fn switch_panel_labels_read_like_a_small_vpn_control() {
        let labels = super::switch_panel_labels(super::SwitchPanelSnapshot {
            product: "Qorx Void".to_string(),
            version: "0.0.1-ylem".to_string(),
            enabled: true,
            kept_tokens: 488_837_426,
            sent_tokens: 96_079,
            reduction_x: 5089.0,
            avoided_usd: 1222.0936,
            boot_enabled: true,
        });

        assert_eq!(labels.title, "Qorx Void");
        assert_eq!(labels.state, "Status: On");
        assert_eq!(labels.toggle, "Turn Qorx Off");
        assert_eq!(labels.kept, "Kept here: 488.84M");
        assert_eq!(labels.sent, "Sent to AI: 96.1K");
        assert_eq!(labels.reduction, "Reduction: 5089x");
        assert_eq!(labels.avoided, "Avoided input cost: $1222.09");
    }

    #[test]
    fn tray_icon_click_opens_native_switch_panel_not_browser_monitor() {
        assert_eq!(
            super::tray_icon_click_action(),
            super::TrayIconAction::OpenNativeSwitchPanel
        );
    }

    #[test]
    fn switch_panel_hit_test_makes_the_switch_primary() {
        let layout = super::SwitchPanelLayout::default();

        assert_eq!(
            layout.hit_test(180, 168),
            super::SwitchPanelAction::ToggleVoid
        );
        assert_eq!(
            layout.hit_test(70, 438),
            super::SwitchPanelAction::OpenMonitor
        );
        assert_eq!(layout.hit_test(12, 12), super::SwitchPanelAction::None);
    }

    #[test]
    fn footer_actions_open_monitor_and_workspace() {
        let layout = super::SwitchPanelLayout::default();

        assert_eq!(
            layout.hit_test(70, 438),
            super::SwitchPanelAction::OpenMonitor
        );
        assert_eq!(
            layout.hit_test(240, 438),
            super::SwitchPanelAction::OpenWorkspace
        );
        assert_eq!(
            layout.hit_test(70, 480),
            super::SwitchPanelAction::ToggleBoot
        );
        assert_eq!(
            layout.hit_test(240, 480),
            super::SwitchPanelAction::ToggleTheme
        );
    }

    #[test]
    fn switch_panel_uses_monitor_palettes_and_fonts() {
        let theme = super::SwitchPanelTheme::monitor(super::TrayThemeMode::Light);

        assert_eq!(theme.bg, super::rgb(0xf4, 0xf3, 0xee));
        assert_eq!(theme.paper, super::rgb(0xfb, 0xfa, 0xf5));
        assert_eq!(theme.paper_soft, super::rgb(0xee, 0xed, 0xe6));
        assert_eq!(theme.ink, super::rgb(0x19, 0x18, 0x17));
        assert_eq!(theme.muted, super::rgb(0x5a, 0x55, 0x4e));
        assert_eq!(theme.line, super::rgb(0xd8, 0xd3, 0xc8));
        assert_eq!(theme.good, super::rgb(0x38, 0x6b, 0x4b));
        assert_eq!(theme.ui_font, "Geist");
        assert_eq!(theme.display_font, "Geist");
        assert_eq!(theme.accent_font, "Instrument Serif");
        assert_eq!(theme.accent_fallback_font, "Georgia");

        let dark = super::SwitchPanelTheme::monitor(super::TrayThemeMode::Dark);
        assert_eq!(dark.bg, super::rgb(0x0f, 0x0e, 0x0c));
        assert_eq!(dark.paper, super::rgb(0x17, 0x15, 0x12));
        assert_eq!(dark.ink, super::rgb(0xf7, 0xf3, 0xea));
        assert_eq!(dark.logo, super::rgb(0xf7, 0xf3, 0xea));
        assert_eq!(
            super::TrayThemeMode::Light.toggled(),
            super::TrayThemeMode::Dark
        );
        assert_eq!(super::TrayThemeMode::Dark.action_label(), "Light mode");
    }

    #[test]
    fn private_font_bundle_matches_monitor_font_link() {
        let names = super::private_font_file_names();

        assert!(names.contains(&"Geist[wght].ttf"));
        assert!(names.contains(&"GeistMono[wght].ttf"));
        assert!(names.contains(&"InstrumentSerif-Regular.ttf"));
        assert!(names.contains(&"InstrumentSerif-Italic.ttf"));
    }

    #[test]
    fn tray_click_shows_from_cached_state_before_refreshing() {
        assert_eq!(
            super::switch_panel_open_strategy(),
            super::SwitchPanelOpenStrategy::ShowCachedThenRefreshAsync
        );
    }

    #[test]
    fn switch_panel_has_native_minimize_control() {
        let layout = super::SwitchPanelLayout::default();

        assert_eq!(
            layout.hit_test(288, 30),
            super::SwitchPanelAction::HidePanel
        );
    }

    #[test]
    fn blocking_panel_actions_are_dispatched_off_the_ui_thread() {
        assert_eq!(
            super::panel_action_execution(super::SwitchPanelAction::ToggleVoid),
            super::PanelActionExecution::Background
        );
        assert_eq!(
            super::panel_action_execution(super::SwitchPanelAction::OpenMonitor),
            super::PanelActionExecution::Background
        );
        assert_eq!(
            super::panel_action_execution(super::SwitchPanelAction::OpenWorkspace),
            super::PanelActionExecution::Background
        );
        assert_eq!(
            super::panel_action_execution(super::SwitchPanelAction::ToggleBoot),
            super::PanelActionExecution::Background
        );
        assert_eq!(
            super::panel_action_execution(super::SwitchPanelAction::ToggleTheme),
            super::PanelActionExecution::Immediate
        );
        assert_eq!(
            super::panel_action_execution(super::SwitchPanelAction::HidePanel),
            super::PanelActionExecution::Immediate
        );
    }

    #[test]
    fn popup_position_anchors_to_taskbar_work_area() {
        let monitor = super::rect(0, 0, 1920, 1080);
        let work = super::rect(0, 0, 1920, 1032);
        let tray_click = windows_sys::Win32::Foundation::POINT { x: 1888, y: 1056 };

        assert_eq!(
            super::panel_position_in_work_area(tray_click, monitor, work),
            (1532, 494)
        );
    }

    #[test]
    fn switch_panel_type_scale_stays_compact() {
        let scale = super::SwitchPanelTypeScale::compact();

        assert_eq!(scale.title, -18);
        assert_eq!(scale.claim, -12);
        assert_eq!(scale.body, -12);
        assert_eq!(scale.number, -17);
        assert_eq!(scale.small, -10);
    }

    #[test]
    fn counter_animation_interpolates_visible_numbers() {
        let from = super::SwitchPanelSnapshot {
            product: "Qorx Void".to_string(),
            version: "0.0.1-ylem".to_string(),
            enabled: true,
            kept_tokens: 100,
            sent_tokens: 10,
            reduction_x: 10.0,
            avoided_usd: 1.0,
            boot_enabled: true,
        };
        let to = super::SwitchPanelSnapshot {
            kept_tokens: 300,
            sent_tokens: 30,
            reduction_x: 30.0,
            avoided_usd: 3.0,
            ..from.clone()
        };
        let halfway = super::interpolate_snapshot(&from, &to, 0.5);

        assert_eq!(halfway.kept_tokens, 200);
        assert_eq!(halfway.sent_tokens, 20);
        assert_eq!(halfway.reduction_x, 20.0);
        assert_eq!(halfway.avoided_usd, 2.0);
        assert!(halfway.boot_enabled);
        assert!(super::counters_changed(&from, &to));
    }

    #[test]
    fn panel_repaints_are_not_forced_inside_click_handlers() {
        assert_eq!(
            super::panel_repaint_strategy(),
            super::PanelRepaintStrategy::InvalidateOnly
        );
    }

    #[test]
    fn refresh_thread_posts_ui_work_to_the_message_thread() {
        assert_eq!(
            super::refresh_ui_strategy(),
            super::RefreshUiStrategy::PostMessageToTrayWindow
        );
    }

    #[test]
    fn panel_uses_non_activating_tray_popover_window() {
        assert_eq!(
            super::panel_activation_strategy(),
            super::PanelActivationStrategy::TopmostNoActivate
        );
    }

    #[test]
    fn tray_click_accepts_shell_select_events_too() {
        assert!(super::tray_icon_event(
            windows_sys::Win32::UI::Shell::NIN_SELECT as isize
        ));
        assert!(super::tray_icon_event(
            windows_sys::Win32::UI::WindowsAndMessaging::WM_LBUTTONDBLCLK as isize
        ));
    }

    #[test]
    fn tray_click_reads_the_low_word_notification_code() {
        let packed_select = ((7_isize) << 16) | windows_sys::Win32::UI::Shell::NIN_SELECT as isize;
        let packed_left_click =
            ((9_isize) << 16) | windows_sys::Win32::UI::WindowsAndMessaging::WM_LBUTTONUP as isize;

        assert!(super::tray_icon_event(packed_select));
        assert!(super::tray_icon_event(packed_left_click));
    }

    #[test]
    fn icon_candidates_cover_cargo_bin_and_manifest_layouts() {
        let candidates = super::qorx_icon_candidates(Some(std::path::Path::new(
            r"C:\Users\Marvin\.cargo\bin\qorx.exe",
        )));

        assert!(candidates.contains(&std::path::PathBuf::from(
            r"C:\Users\Marvin\.cargo\bin\icon\qorx-ico.ico"
        )));
        assert!(candidates.contains(&std::path::PathBuf::from(
            r"C:\Users\Marvin\.cargo\icon\qorx-ico.ico"
        )));
        assert!(candidates.contains(&std::path::PathBuf::from(
            r"C:\Users\Marvin\icon\qorx-ico.ico"
        )));
        assert!(candidates
            .iter()
            .any(|path| path.ends_with(std::path::Path::new("icon").join("qorx-ico.ico"))));
    }
}
