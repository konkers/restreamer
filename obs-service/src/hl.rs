use anyhow::{anyhow, Result};
pub use obs as ll;
use std::{
    convert::TryInto,
    ffi::{c_void, CString},
    ptr::{null, null_mut},
};
use x11::{glx, xlib};

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        CString::new($s)?.as_ptr()
    };
}

pub struct Data {
    data: *mut ll::obs_data_t,
}

impl Data {
    pub fn new() -> Result<Data> {
        unsafe {
            let data = ll::obs_data_create();
            if data.is_null() {
                Err(anyhow!("Failed to allocate obs data"))
            } else {
                Ok(Data { data })
            }
        }
    }

    pub fn from_raw(data: *mut ll::obs_data_t) -> Data {
        Data { data }
    }

    pub fn from_json(json_string: &str) -> Result<Data> {
        unsafe {
            let data = ll::obs_data_create_from_json(CString::new(json_string)?.as_ptr());
            if data.is_null() {
                Err(anyhow!("failed to crate data from json"))
            } else {
                Ok(Data { data })
            }
        }
    }

    pub fn set_string(&mut self, key: &str, val: &str) -> Result<()> {
        unsafe {
            let key = CString::new(key)?;
            let val = CString::new(val)?;
            ll::obs_data_set_string(self.data, key.as_ptr(), val.as_ptr());
        }
        Ok(())
    }

    pub fn set_int(&mut self, key: &str, val: i64) -> Result<()> {
        unsafe {
            let key = CString::new(key)?;
            ll::obs_data_set_int(self.data, key.as_ptr(), val);
        }
        Ok(())
    }

    pub fn set_bool(&mut self, key: &str, val: bool) -> Result<()> {
        unsafe {
            let key = CString::new(key)?;
            ll::obs_data_set_bool(self.data, key.as_ptr(), val);
        }
        Ok(())
    }

    pub fn set_object(&mut self, key: &str, mut data: Data) -> Result<()> {
        unsafe {
            let key = CString::new(key)?;
            ll::obs_data_set_obj(self.data, key.as_ptr(), data.as_mut_ptr());
        }
        Ok(())
    }

    pub fn set_array(&mut self, key: &str, mut data: Array) -> Result<()> {
        unsafe {
            let key = CString::new(key)?;
            ll::obs_data_set_array(self.data, key.as_ptr(), data.as_mut_ptr());
        }
        Ok(())
    }

    pub fn get_array(&mut self, key: &str) -> Result<Array> {
        unsafe {
            let key = CString::new(key)?;
            let data = ll::obs_data_get_array(self.data, key.as_ptr());
            if data.is_null() {
                Err(anyhow!("array {:?} does not exist", key))
            } else {
                Ok(Array::from_raw(data))
            }
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut ll::obs_data_t {
        self.data
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        unsafe {
            ll::obs_data_release(self.data);
        }
    }
}

pub struct Array {
    data: *mut ll::obs_data_array_t,
}

impl Array {
    pub fn new() -> Result<Array> {
        unsafe {
            let data = ll::obs_data_array_create();
            if data.is_null() {
                Err(anyhow!("failed to allocate array"))
            } else {
                Ok(Array { data })
            }
        }
    }

    pub fn from_raw(data: *mut ll::obs_data_array_t) -> Array {
        Array { data }
    }

    pub fn push_back(&mut self, mut data: Data) {
        unsafe {
            ll::obs_data_array_push_back(self.data, data.as_mut_ptr());
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut ll::obs_data_array_t {
        self.data
    }
}

impl Drop for Array {
    fn drop(&mut self) {
        unsafe {
            ll::obs_data_array_release(self.data);
        }
    }
}

pub struct SessionSettings {
    pub base_width: u32,
    pub base_height: u32,
    pub output_width: u32,
    pub output_height: u32,
    pub fps: u32,
    pub audio_samples_per_sec: u32,
    pub stream_url: String,
    pub stream_key: String,
}

pub struct Session {
    output: *mut obs::obs_output_t,
    service: *mut obs::obs_service_t,
    audio_encoder: *mut obs::obs_encoder_t,
    video_encoder: *mut obs::obs_encoder_t,
}

impl Session {
    pub fn new(settings: &SessionSettings) -> Result<Session> {
        let display_env = std::env::var("DISPLAY")?;
        unsafe {
            let d = xlib::XOpenDisplay(CString::new(display_env)?.as_ptr());
            let mut a = 0;
            let mut b = 0;

            let ret = glx::glXQueryExtension(d, &mut a as _, &mut b as _);
            if ret == 0 {
                return Err(anyhow!("query extension failed"));
            }
            if false {
                let mut opts = vec![
                    glx::GLX_RGBA,
                    glx::GLX_RED_SIZE,
                    1,
                    glx::GLX_GREEN_SIZE,
                    1,
                    glx::GLX_BLUE_SIZE,
                    1,
                    glx::GLX_DEPTH_SIZE,
                    12,
                    glx::GLX_DOUBLEBUFFER,
                    0,
                ];

                let visual = glx::glXChooseVisual(d, xlib::XDefaultScreen(d), opts.as_mut_ptr());
                if visual.is_null() {
                    return Err(anyhow!("can't choose visual"));
                }
            }

            obs::obs_set_nix_platform_display(d as *mut c_void);
            let ret = obs::obs_startup(CString::new("en-US")?.as_ptr(), null(), null_mut());
            if !ret {
                return Err(anyhow!("obs_startup failed"));
            }

            if !obs::obs_initialized() {
                return Err(anyhow!("obs not initialized after startup"));
            }

            let module = CString::new("/usr/lib/libobs-opengl.so.0.0")?;
            let mut video_info = obs::obs_video_info {
                adapter: 0,
                graphics_module: module.as_ptr(),
                output_format: obs::video_format_VIDEO_FORMAT_NV12,
                fps_num: settings.fps,
                fps_den: 1,
                base_width: settings.base_width,
                base_height: settings.base_height,
                output_width: settings.output_width,
                output_height: settings.output_height,
                gpu_conversion: true,
                colorspace: 0,
                range: 0,
                scale_type: 0,
            };

            let err = obs::obs_reset_video((&mut video_info) as _);
            if err != obs::OBS_VIDEO_SUCCESS as i32 {
                return Err(anyhow!("obs reset video failed: {}", err));
            }

            let mut audio_info = obs::obs_audio_info {
                samples_per_sec: settings.audio_samples_per_sec,
                speakers: 2,
            };
            let err = obs::obs_reset_audio((&mut audio_info) as _);
            if err != true {
                return Err(anyhow!("obs reset audio failed: {}", err));
            }

            load_module("image-source")?;
            load_module("obs-ffmpeg")?;
            load_module("obs-transitions")?;
            load_module("rtmp-services")?;
            load_module("obs-x264")?;
            load_module("obs-libfdk")?;
            load_module("obs-outputs")?;
            load_module("vlc-video")?;
            load_module("obs-browser")?;

            obs::obs_post_load_modules();

            let service = obs::obs_service_create(
                CString::new("rtmp_common")?.as_ptr(),
                CString::new("rtmp service")?.as_ptr(),
                null_mut(),
                null_mut(),
            );

            if service.is_null() {
                return Err(anyhow!("create service failed"));
            }

            let mut rtmp_settings = Data::new()?;
            rtmp_settings.set_string("server", &settings.stream_url)?;
            rtmp_settings.set_string("key", &settings.stream_key)?;
            obs::obs_service_update(service, rtmp_settings.as_mut_ptr());

            let output = obs::obs_output_create(
                CString::new("rtmp_output")?.as_ptr(),
                CString::new("RTMP output")?.as_ptr(),
                null_mut(),
                null_mut(),
            );
            if output.is_null() {
                return Err(anyhow!("create output failed"));
            }

            let audio_encoder = obs::obs_audio_encoder_create(
                cstr!("libfdk_aac"),
                cstr!("aac enc"),
                null_mut(),
                0,
                null_mut(),
            );
            if audio_encoder.is_null() {
                return Err(anyhow!("failed to create audio encoder"));
            }

            let settings = obs::obs_encoder_get_settings(audio_encoder);
            if settings.is_null() {
                return Err(anyhow!("failed to get encoder settings"));
            }
            let mut settings = Data::from_raw(settings);
            settings.set_int("bitrate", 48000)?;
            settings.set_bool("afterburner", true)?;
            obs::obs_encoder_update(audio_encoder, settings.as_mut_ptr());

            let video_encoder = obs::obs_video_encoder_create(
                cstr!("obs_x264"),
                cstr!("h264 enc"),
                null_mut(),
                null_mut(),
            );
            if video_encoder.is_null() {
                return Err(anyhow!("failed to create video encoder"));
            }

            let settings = obs::obs_encoder_get_settings(video_encoder);
            if settings.is_null() {
                return Err(anyhow!("failed to get video settings"));
            }
            let mut settings = Data::from_raw(settings);
            settings.set_int("bitrate", 2000)?;
            settings.set_int("keyint_sec", 2)?;
            settings.set_string("rate_control", "CBR")?;

            // These are settings for the sw decoder
            settings.set_int("width", video_info.output_width as i64)?;
            settings.set_int("height", video_info.output_height as i64)?;
            settings.set_int("fps_num", video_info.fps_num as i64)?;
            settings.set_int("fps_den", video_info.fps_den as i64)?;
            settings.set_string("preset", "ultrafast")?;
            settings.set_string("profile", "main")?;
            settings.set_string("tune", "zerolatency")?;
            settings.set_string("x264opts", "")?;
            obs::obs_encoder_update(video_encoder, settings.as_mut_ptr());

            Ok(Session {
                output,
                service,
                audio_encoder,
                video_encoder,
            })
        }
    }

    pub fn load_config_json(&mut self, json_str: &str) -> Result<()> {
        unsafe {
            let mut data = Data::from_json(&json_str)?;

            let mut sources = data.get_array("sources")?;
            let files = obs::obs_missing_files_create();
            obs::obs_load_sources(sources.as_mut_ptr(), Some(sources_cb), files as _);

            let scene = obs::obs_get_source_by_name(cstr!("restream"));
            if scene.is_null() {
                return Err(anyhow!("can't get restream source"));
            }
            obs::obs_set_output_source(0, scene);
            obs::obs_encoder_set_video(self.video_encoder, obs::obs_get_video());
            obs::obs_encoder_set_audio(self.audio_encoder, obs::obs_get_audio());
            obs::obs_output_set_video_encoder(self.output, self.video_encoder);
            obs::obs_output_set_audio_encoder(self.output, self.audio_encoder, 0);
            obs::obs_output_set_service(self.output, self.service);
        }

        Ok(())
    }

    pub fn start(self) -> Result<()> {
        unsafe {
            if obs::obs_output_start(self.output) != true {
                return Err(anyhow!("output start failed"));
            }
        }

        Ok(())
    }
}

unsafe extern "C" fn sources_cb(param: *mut c_void, source: *mut obs::obs_source_t) {
    // If we don't take a ref here, the sources disappear.
    obs::obs_source_addref(source);
    let f = param as *mut obs::obs_missing_files_t;
    let sf = obs::obs_source_get_missing_files(source);

    obs::obs_missing_files_append(f, sf);
    obs::obs_missing_files_destroy(sf);
}

unsafe fn load_module(module: &str) -> Result<()> {
    let bin_path = CString::new(format!("/usr/lib/obs-plugins/{}.so", module))?;
    let data_path = CString::new(format!("/usr/share/obs/obs-plugins/{}", module))?;
    let mut module = null_mut();
    let ret = obs::obs_open_module(&mut module as _, bin_path.as_ptr(), data_path.as_ptr());
    if ret != obs::MODULE_SUCCESS as _ {
        return Err(anyhow!(
            "unable to open obs module {:?}:{:?}",
            bin_path,
            data_path
        ));
    }

    if !obs::obs_init_module(module) {
        return Err(anyhow!(
            "unable to open init module {:?}:{:?}",
            bin_path,
            data_path
        ));
    }
    Ok(())
}
