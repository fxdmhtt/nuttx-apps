#include <nuttx/config.h>

// #include <stdio.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/unistd.h>
#include <lvgl/src/libs/fsdrv/lv_fsdrv.h>

#ifdef main
#undef main
#endif
#include <lvgl/lvgl.h>

static void *vm = NULL;
void *game2048_new(void);
void game2048_drop(void *);

static void game2048_start(void)
{
    mkdir("/game2048", 0755);
    mount(NULL, "/game2048", "hostfs", 0, "fs=./resources/game2048");
    lv_fs_posix_init();
}

static void game2048_resume(void)
{
    vm = game2048_new();
}

static void game2048_pause(void)
{
    game2048_drop(vm);
}

static void game2048_stop(void)
{
    umount("/game2048");
    rmdir("/game2048");
}

static void game2048_launcher(lv_event_t *e)
{
    lv_obj_t *lbl = lv_event_get_user_data(e);

    if (lv_strcmp(lv_label_get_text(lbl), "Start Game2048 demo") == 0)
    {
        game2048_start();
        game2048_resume();

        lv_label_set_text(lbl, "Stop Game2048 demo");
    }
    else
    {
        game2048_pause();
        game2048_stop();

        lv_label_set_text(lbl, "Start Game2048 demo");
    }
}

void game2048_main(void)
{
    lv_obj_t *btn = lv_button_create(lv_screen_active());
    lv_obj_align(btn, LV_ALIGN_TOP_RIGHT, 0, 0);
    lv_obj_t *lbl = lv_label_create(btn);
    lv_label_set_text(lbl, "Start Game2048 demo");
    lv_obj_add_event_cb(btn, game2048_launcher, LV_EVENT_SHORT_CLICKED, lbl);
}
