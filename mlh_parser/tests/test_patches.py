from mlh_parser.parser_algorithm import extract_patches

input_syzbot_mail = """
syzbot has found a reproducer for the following crash on:

HEAD commit:    79c3ba32 Merge tag 'drm-fixes-2019-06-07-1' of git://anong..
git tree:       upstream
console output: https://syzkaller.appspot.com/x/log.txt?x=1201b971a00000
kernel config:  https://syzkaller.appspot.com/x/.config?x=60564cb52ab29d5b
dashboard link: https://syzkaller.appspot.com/bug?extid=2ff1e7cb738fd3c41113
compiler:       gcc (GCC) 9.0.0 20181231 (experimental)
syz repro:      https://syzkaller.appspot.com/x/repro.syz?x=14a3bf51a00000
C reproducer:   https://syzkaller.appspot.com/x/repro.c?x=120d19f2a00000

The bug was bisected to:

commit 0fff724a33917ac581b5825375d0b57affedee76
Author: Paul Kocialkowski <paul.kocialkowski@bootlin.com>
Date:   Fri Jan 18 14:51:13 2019 +0000

     drm/sun4i: backend: Use explicit fourcc helpers for packed YUV422 check

bisection log:  https://syzkaller.appspot.com/x/bisect.txt?x=1467550f200000
final crash:    https://syzkaller.appspot.com/x/report.txt?x=1667550f200000
console output: https://syzkaller.appspot.com/x/log.txt?x=1267550f200000

IMPORTANT: if you fix the bug, please add the following tag to the commit:
Reported-by: syzbot+2ff1e7cb738fd3c41113@syzkaller.appspotmail.com
Fixes: 0fff724a3391 ("drm/sun4i: backend: Use explicit fourcc helpers for  
packed YUV422 check")

WARNING: CPU: 0 PID: 8951 at kernel/bpf/core.c:851 bpf_jit_free+0x157/0x1b0
Kernel panic - not syncing: panic_on_warn set ...
CPU: 0 PID: 8951 Comm: kworker/0:0 Not tainted 5.2.0-rc3+ #23
Hardware name: Google Google Compute Engine/Google Compute Engine, BIOS  
Google 01/01/2011
Workqueue: events bpf_prog_free_deferred
Call Trace:
  __dump_stack lib/dump_stack.c:77 [inline]
  dump_stack+0x172/0x1f0 lib/dump_stack.c:113
  panic+0x2cb/0x744 kernel/panic.c:219
  __warn.cold+0x20/0x4d kernel/panic.c:576
  report_bug+0x263/0x2b0 lib/bug.c:186
  fixup_bug arch/x86/kernel/traps.c:179 [inline]
  fixup_bug arch/x86/kernel/traps.c:174 [inline]
  do_error_trap+0x11b/0x200 arch/x86/kernel/traps.c:272
  do_invalid_op+0x37/0x50 arch/x86/kernel/traps.c:291
  invalid_op+0x14/0x20 arch/x86/entry/entry_64.S:986
RIP: 0010:bpf_jit_free+0x157/0x1b0
Code: 00 fc ff df 48 89 fa 48 c1 ea 03 80 3c 02 00 75 5d 48 b8 00 02 00 00  
00 00 ad de 48 39 43 70 0f 84 05 ff ff ff e8 f9 b5 f4 ff <0f> 0b e9 f9 fe  

"""


def test_syzbot_email() -> None:
    output = extract_patches(input_syzbot_mail)
    assert len(output) == 0


input_example_mail = """

Interfaces for when a new domain in the crashdump kernel needs some
values from the panicked kernel's context entries.

Signed-off-by: Bill Sumner 
---
 drivers/iommu/intel-iommu.c | 46 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)

diff --git a/drivers/iommu/intel-iommu.c b/drivers/iommu/intel-iommu.c
index 73afed4..bde8f22 100644
--- a/drivers/iommu/intel-iommu.c
+++ b/drivers/iommu/intel-iommu.c
@@ -348,6 +348,7 @@ static inline int first_pte_in_page(struct dma_pte *pte)
 
 
 #ifdef CONFIG_CRASH_DUMP
+
 /*
  * Fix Crashdump failure caused by leftover DMA through a hardware IOMMU
  *
@@ -437,6 +438,16 @@ static int intel_iommu_copy_translation_tables(struct dmar_drhd_unit *drhd,
                struct root_entry **root_new_virt_p,
                int g_num_of_iommus);
 
+static struct domain_values_entry *intel_iommu_did_to_domain_values_entry(
+               int did, struct intel_iommu *iommu);
+
+static int intel_iommu_get_dids_from_old_kernel(struct intel_iommu *iommu);
+
+static struct domain_values_entry *intel_iommu_did_to_domain_values_entry(
+               int did, struct intel_iommu *iommu);
+
+static int intel_iommu_get_dids_from_old_kernel(struct intel_iommu *iommu);
+
 #endif /* CONFIG_CRASH_DUMP */
 
 /*
@@ -5230,4 +5241,39 @@ static int intel_iommu_copy_translation_tables(struct dmar_drhd_unit *drhd,
        return 0;
 }
 
+/*
+ * Interfaces for when a new domain in the crashdump kernel needs some
+ * values from the panicked kernel's context entries
+ *
+ */
+static struct domain_values_entry *intel_iommu_did_to_domain_values_entry(
+               int did, struct intel_iommu *iommu)
+{
+       struct domain_values_entry *dve;        /* iterator */
+
+       list_for_each_entry(dve, &domain_values_list[iommu->seq_id], link)
+               if (dve->did == did)
+                       return dve;
+       return NULL;
+}
+
+/* Mark domain-id's from old kernel as in-use on this iommu so that a new
+ * domain-id is allocated in the case where there is a device in the new kernel
+ * that was not in the old kernel -- and therefore a new domain-id is needed.
+ */
+static int intel_iommu_get_dids_from_old_kernel(struct intel_iommu *iommu)
+{
+       struct domain_values_entry *dve;        /* iterator */
+
+       pr_info("IOMMU:%d Domain ids from panicked kernel:\n", iommu->seq_id);
+
+       list_for_each_entry(dve, &domain_values_list[iommu->seq_id], link) {
+               set_bit(dve->did, iommu->domain_ids);
+               pr_info("DID did:%d(0x%4.4x)\n", dve->did, dve->did);
+       }
+
+       pr_info("----------------------------------------\n");
+       return 0;
+}
+
 #endif /* CONFIG_CRASH_DUMP */
-- 
2.0.0-rc0

""".strip()


diff = [
    """---
 drivers/iommu/intel-iommu.c | 46 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)

diff --git a/drivers/iommu/intel-iommu.c b/drivers/iommu/intel-iommu.c
index 73afed4..bde8f22 100644
--- a/drivers/iommu/intel-iommu.c
+++ b/drivers/iommu/intel-iommu.c
@@ -348,6 +348,7 @@ static inline int first_pte_in_page(struct dma_pte *pte)
 
 
 #ifdef CONFIG_CRASH_DUMP
+
 /*
  * Fix Crashdump failure caused by leftover DMA through a hardware IOMMU
  *
@@ -437,6 +438,16 @@ static int intel_iommu_copy_translation_tables(struct dmar_drhd_unit *drhd,
                struct root_entry **root_new_virt_p,
                int g_num_of_iommus);
 
+static struct domain_values_entry *intel_iommu_did_to_domain_values_entry(
+               int did, struct intel_iommu *iommu);
+
+static int intel_iommu_get_dids_from_old_kernel(struct intel_iommu *iommu);
+
+static struct domain_values_entry *intel_iommu_did_to_domain_values_entry(
+               int did, struct intel_iommu *iommu);
+
+static int intel_iommu_get_dids_from_old_kernel(struct intel_iommu *iommu);
+
 #endif /* CONFIG_CRASH_DUMP */
 
 /*
@@ -5230,4 +5241,39 @@ static int intel_iommu_copy_translation_tables(struct dmar_drhd_unit *drhd,
        return 0;
 }
 
+/*
+ * Interfaces for when a new domain in the crashdump kernel needs some
+ * values from the panicked kernel's context entries
+ *
+ */
+static struct domain_values_entry *intel_iommu_did_to_domain_values_entry(
+               int did, struct intel_iommu *iommu)
+{
+       struct domain_values_entry *dve;        /* iterator */
+
+       list_for_each_entry(dve, &domain_values_list[iommu->seq_id], link)
+               if (dve->did == did)
+                       return dve;
+       return NULL;
+}
+
+/* Mark domain-id's from old kernel as in-use on this iommu so that a new
+ * domain-id is allocated in the case where there is a device in the new kernel
+ * that was not in the old kernel -- and therefore a new domain-id is needed.
+ */
+static int intel_iommu_get_dids_from_old_kernel(struct intel_iommu *iommu)
+{
+       struct domain_values_entry *dve;        /* iterator */
+
+       pr_info("IOMMU:%d Domain ids from panicked kernel:\n", iommu->seq_id);
+
+       list_for_each_entry(dve, &domain_values_list[iommu->seq_id], link) {
+               set_bit(dve->did, iommu->domain_ids);
+               pr_info("DID did:%d(0x%4.4x)\n", dve->did, dve->did);
+       }
+
+       pr_info("----------------------------------------\n");
+       return 0;
+}
+
 #endif /* CONFIG_CRASH_DUMP */
-- 
2.0.0-rc0
""".strip(),
]


def test_corret_email() -> None:
    output = extract_patches(input_example_mail)
    assert output == diff


# --- Test Input Definitions ---

# Test Case 1: No patch content
input_syzbot_mail = """
syzbot has found a reproducer for the following crash on:

HEAD commit:    79c3ba32 Merge tag 'drm-fixes-2019-06-07-1' of git://anong..
git tree:       upstream
console output: https://syzkaller.appspot.com/x/log.txt?x=1201b971a00000
kernel config:  https://syzkaller.appspot.com/x/.config?x=60564cb52ab29d5b
dashboard link: https://syzkaller.appspot.com/bug?extid=2ff1e7cb738fd3c41113
compiler:       gcc (GCC) 9.0.0 20181231 (experimental)
syz repro:      https://syzkaller.appspot.com/x/repro.syz?x=14a3bf51a00000
C reproducer:   https://syzkaller.appspot.com/x/repro.c?x=120d19f2a00000
"""


input_single_patch_mail = """
Interfaces for when a new domain in the crashdump kernel needs some
values from the panicked kernel's context entries.

Signed-off-by: Bill Sumner 
---
 drivers/iommu/intel-iommu.c | 46 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)

diff --git a/drivers/iommu/intel-iommu.c b/drivers/iommu/intel-iommu.c
index 73afed4..bde8f22 100644
--- a/drivers/iommu/intel-iommu.c
+++ b/drivers/iommu/intel-iommu.c
@@ -348,6 +348,7 @@ static inline int first_pte_in_page(struct dma_pte *pte)
#endif /* CONFIG_CRASH_DUMP */
--
2.0.0-rc0

""".strip()

expected_single_patch = [
    """---
 drivers/iommu/intel-iommu.c | 46 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)

diff --git a/drivers/iommu/intel-iommu.c b/drivers/iommu/intel-iommu.c
index 73afed4..bde8f22 100644
--- a/drivers/iommu/intel-iommu.c
+++ b/drivers/iommu/intel-iommu.c
@@ -348,6 +348,7 @@ static inline int first_pte_in_page(struct dma_pte *pte)
#endif /* CONFIG_CRASH_DUMP */
--
2.0.0-rc0
""".strip(),
]

input_multiple_patches_mail = """
This is the first patch in the series. It fixes a minor typo.
Signed-off-by: Joe Developer <joe@example.com>
---
 drivers/net/ethernet/intel/e1000/e1000_main.c | 1 +
 1 file changed, 1 insertion(+)

diff --git a/drivers/net/ethernet/intel/e1000/e1000_main.c b/drivers/net/ethernet/intel/e1000/e1000_main.c
index a2d9b23..b3f1c40 100644
--- a/drivers/net/ethernet/intel/e1000/e1000_main.c
+++ b/drivers/net/ethernet/intel/e1000/e1000_main.c
@@ -512,6 +512,7 @@ static void e1000_reset_task(struct work_struct *work)
     /* Fix the typo in the previous comment */
+     /* New line of code */
     e1000_reset(adapter);
 }
-- 
2.30.0


This is the second patch. It adds a function.
Signed-off-by: Jane Developer <jane@example.com>
---
 include/linux/random.h | 11 +++++++++++
 1 file changed, 11 insertions(+)

diff --git a/include/linux/random.h b/include/linux/random.h
index e2c9a1d..f8e3f4c 100644
--- a/include/linux/random.h
+++ b/include/linux/random.h
@@ -5,6 +5,17 @@
 #include <linux/types.h>
 
+/**
+ * pr_get_random_int - Get a cryptographically secure random integer
+ *
+ * Returns a 32-bit random integer.
+ */
+static inline u32 pr_get_random_int(void)
+{
+    return get_random_u32();
+}
+
 extern void get_random_bytes(void *buf, int nbytes);
 extern void get_random_bytes_arch(void *buf, int nbytes);
-- 
2.30.0

""".strip()

expected_multiple_patches = [
    # patch 1
    """---
 drivers/net/ethernet/intel/e1000/e1000_main.c | 1 +
 1 file changed, 1 insertion(+)

diff --git a/drivers/net/ethernet/intel/e1000/e1000_main.c b/drivers/net/ethernet/intel/e1000/e1000_main.c
index a2d9b23..b3f1c40 100644
--- a/drivers/net/ethernet/intel/e1000/e1000_main.c
+++ b/drivers/net/ethernet/intel/e1000/e1000_main.c
@@ -512,6 +512,7 @@ static void e1000_reset_task(struct work_struct *work)
     /* Fix the typo in the previous comment */
+     /* New line of code */
     e1000_reset(adapter);
 }
-- 
2.30.0
""".strip(),
    # patch 2
    """---
 include/linux/random.h | 11 +++++++++++
 1 file changed, 11 insertions(+)

diff --git a/include/linux/random.h b/include/linux/random.h
index e2c9a1d..f8e3f4c 100644
--- a/include/linux/random.h
+++ b/include/linux/random.h
@@ -5,6 +5,17 @@
 #include <linux/types.h>
 
+/**
+ * pr_get_random_int - Get a cryptographically secure random integer
+ *
+ * Returns a 32-bit random integer.
+ */
+static inline u32 pr_get_random_int(void)
+{
+    return get_random_u32();
+}
+
 extern void get_random_bytes(void *buf, int nbytes);
 extern void get_random_bytes_arch(void *buf, int nbytes);
-- 
2.30.0
""".strip(),
]


input_single_patch_mail_without_space_before_git_version = """
Interfaces for when a new domain in the crashdump kernel needs some
values from the panicked kernel's context entries.

Signed-off-by: Bill Sumner 
---
 drivers/iommu/intel-iommu.c | 46 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)

diff --git a/drivers/iommu/intel-iommu.c b/drivers/iommu/intel-iommu.c
index 73afed4..bde8f22 100644
--- a/drivers/iommu/intel-iommu.c
+++ b/drivers/iommu/intel-iommu.c
@@ -348,6 +348,7 @@ static inline int first_pte_in_page(struct dma_pte *pte)
#endif /* CONFIG_CRASH_DUMP */
--
2.0.0-rc0

""".strip()


expected_patch_mail_without_space_before_git_version = [
    """
---
 drivers/iommu/intel-iommu.c | 46 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)

diff --git a/drivers/iommu/intel-iommu.c b/drivers/iommu/intel-iommu.c
index 73afed4..bde8f22 100644
--- a/drivers/iommu/intel-iommu.c
+++ b/drivers/iommu/intel-iommu.c
@@ -348,6 +348,7 @@ static inline int first_pte_in_page(struct dma_pte *pte)
#endif /* CONFIG_CRASH_DUMP */
--
2.0.0-rc0
""".strip()
]


input_single_patch_mail_with_many_spaces_before_git_version = """
Interfaces for when a new domain in the crashdump kernel needs some
values from the panicked kernel's context entries.

Signed-off-by: Bill Sumner 
---
 drivers/iommu/intel-iommu.c | 46 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)

diff --git a/drivers/iommu/intel-iommu.c b/drivers/iommu/intel-iommu.c
index 73afed4..bde8f22 100644
--- a/drivers/iommu/intel-iommu.c
+++ b/drivers/iommu/intel-iommu.c
@@ -348,6 +348,7 @@ static inline int first_pte_in_page(struct dma_pte *pte)
#endif /* CONFIG_CRASH_DUMP */
--              
2.0.0-rc0

""".strip()


expected_patch_mail_with_many_spaces_before_git_version = [
    """
---
 drivers/iommu/intel-iommu.c | 46 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 46 insertions(+)

diff --git a/drivers/iommu/intel-iommu.c b/drivers/iommu/intel-iommu.c
index 73afed4..bde8f22 100644
--- a/drivers/iommu/intel-iommu.c
+++ b/drivers/iommu/intel-iommu.c
@@ -348,6 +348,7 @@ static inline int first_pte_in_page(struct dma_pte *pte)
#endif /* CONFIG_CRASH_DUMP */
--              
2.0.0-rc0
""".strip()
]


# --- Test Functions ---


def test_no_patch_in_email() -> None:
    """Tests an email that clearly contains no patches."""
    output = extract_patches(input_syzbot_mail)
    assert len(output) == 0
    assert output == []


def test_single_patch_full_format() -> None:
    """Tests a single patch email with diffstat and the final separator."""
    output = extract_patches(input_single_patch_mail)
    assert len(output) == 1

    assert output == expected_single_patch


def test_multiple_patches_in_email() -> None:
    """Tests an email containing two patches separated by regular text."""
    output = extract_patches(input_multiple_patches_mail)
    assert len(output) == 2

    # Patch 1 check
    assert output == expected_multiple_patches


def test_mail_without_space_before_git_version() -> None:
    """Tests a single patch email with diffstat and the final separator."""
    output = extract_patches(input_single_patch_mail_without_space_before_git_version)
    assert len(output) == 1

    assert output == expected_patch_mail_without_space_before_git_version


def test_mail_with_many_spaces_before_git_version() -> None:
    """Tests a single patch email with diffstat and the final separator."""
    output = extract_patches(
        input_single_patch_mail_with_many_spaces_before_git_version
    )
    assert len(output) == 1
    expected_diff = expected_patch_mail_with_many_spaces_before_git_version

    assert output == expected_diff


#

# a few real mails

# schema: [{"mail": "body", "expected": ["patch1", ... "patchN"]}]
real_mails = [
    {
        "mail": """--===============1184715646109867569==
Content-Type: text/plain; charset="utf-8"
MIME-Version: 1.0
Content-Transfer-Encoding: quoted-printable

From: Ramesh Thomas <ramesh.thomas(a)intel.com>

Add Intel Analytics Accelerator support.

Changes in v2:
- block on fault is available for IAX

Signed-off-by: Ramesh Thomas <ramesh.thomas(a)intel.com>
Signed-off-by: Dave Jiang <dave.jiang(a)intel.com>
---
 accfg/config.c          | 10 ++++++++++
 accfg/lib/libaccfg.c    |  3 +++
 accfg/libaccel_config.h |  1 +
 accfg/list.c            | 31 +++++++++++++++++++------------
 util/json.c             | 16 ++++++++++++----
 5 files changed, 45 insertions(+), 16 deletions(-)

diff --git a/accfg/config.c b/accfg/config.c
index 3887873..d36f1d9 100644
--- a/accfg/config.c
+++ b/accfg/config.c
@@ -296,6 +296,10 @@ static int group_json_set_val(struct accfg_group *grou=
p,
                json_object *jobj, char *key)
 {
        int rc, i;
+       struct accfg_device *dev =3D NULL;
+
+       if (group)
+               dev =3D accfg_group_get_device(group);
 =

        if (!group || !jobj || !key)
                return -EINVAL;
@@ -311,6 +315,12 @@ static int group_json_set_val(struct accfg_group *grou=
p,
                                                || (val < 0))
                                        return -EINVAL;
 =

+                               if ((accfg_device_get_type(dev) =3D=3D ACCFG_DEVICE_IAX)
+                                       && ((!strcmp(group_table[i].name, "tokens_reserved"))
+                                       || (!strcmp(group_table[i].name, "use_token_limit"))
+                                       || (!strcmp(group_table[i].name, "tokens_allowed")))) {
+                                       return 0;
+                               }
                                if (group_table[i].is_writable &&
                                        !group_table[i].is_writable(group,
                                                val))
diff --git a/accfg/lib/libaccfg.c b/accfg/lib/libaccfg.c
index 70553f7..cd9da42 100644
--- a/accfg/lib/libaccfg.c
+++ b/accfg/lib/libaccfg.c
@@ -39,6 +39,7 @@ static int filename_prefix_len;
 =

 ACCFG_EXPORT char *accfg_basenames[] =3D {
         [ACCFG_DEVICE_DSA]      =3D "dsa",
+       [ACCFG_DEVICE_IAX]      =3D "iax",
        NULL
 };
 =

@@ -397,6 +398,8 @@ static int device_parse_type(struct accfg_device *devic=
e)
 =

        if (!strcmp(device->device_type_str, "dsa"))
                device->type =3D ACCFG_DEVICE_DSA;
+       else if (!strcmp(device->device_type_str, "iax"))
+               device->type =3D ACCFG_DEVICE_IAX;
        else
                device->type =3D ACCFG_DEVICE_TYPE_UNKNOWN;
 =

diff --git a/accfg/libaccel_config.h b/accfg/libaccel_config.h
index 392188e..f85670c 100644
--- a/accfg/libaccel_config.h
+++ b/accfg/libaccel_config.h
@@ -31,6 +31,7 @@ extern "C" {
 /* no need to save device state */
 enum accfg_device_type {
        ACCFG_DEVICE_DSA =3D 0,
+       ACCFG_DEVICE_IAX =3D 1,
        ACCFG_DEVICE_TYPE_UNKNOWN =3D -1,
 };
 =

diff --git a/accfg/list.c b/accfg/list.c
index dfaac1f..c22da41 100644
--- a/accfg/list.c
+++ b/accfg/list.c
@@ -54,6 +54,10 @@ static struct json_object *group_to_json(struct accfg_gr=
oup *group,
 {
        struct json_object *jgroup =3D json_object_new_object();
        struct json_object *jobj =3D NULL;
+       struct accfg_device *dev =3D NULL;
+
+       if (group)
+               dev =3D accfg_group_get_device(group);
 =

        if (!jgroup)
                return NULL;
@@ -67,20 +71,23 @@ static struct json_object *group_to_json(struct accfg_g=
roup *group,
        if (!jobj)
                goto err;
 =

-       json_object_object_add(jgroup, "tokens_reserved", jobj);
-       jobj =3D json_object_new_int(accfg_group_get_use_token_limit(group));
-       if (!jobj)
-               goto err;
+       if (accfg_device_get_type(dev) !=3D ACCFG_DEVICE_IAX) {
+               json_object_object_add(jgroup, "tokens_reserved", jobj);
+               jobj =3D json_object_new_int(accfg_group_get_use_token_limit(group));
+               if (!jobj)
+                       goto err;
 =

-       json_object_object_add(jgroup, "use_token_limit", jobj);
-       jobj =3D json_object_new_int(accfg_group_get_tokens_allowed(group));
-       if (!jobj)
-               goto err;
+               json_object_object_add(jgroup, "use_token_limit", jobj);
+               jobj =3D json_object_new_int(accfg_group_get_tokens_allowed(group));
+               if (!jobj)
+                       goto err;
 =

-       json_object_object_add(jgroup, "tokens_allowed", jobj);
-       jobj =3D json_object_new_int(accfg_group_get_traffic_class_a(group));
-       if (!jobj)
-               goto err;
+               json_object_object_add(jgroup, "tokens_allowed", jobj);
+               jobj =3D json_object_new_int(accfg_group_get_traffic_class_a(
+                               group));
+               if (!jobj)
+                       goto err;
+       }
 =

        json_object_object_add(jgroup, "traffic_class_a", jobj);
        jobj =3D json_object_new_int(accfg_group_get_traffic_class_b(
diff --git a/util/json.c b/util/json.c
index 3bf8950..bb3ee88 100644
--- a/util/json.c
+++ b/util/json.c
@@ -178,7 +178,8 @@ struct json_object *util_device_to_json(struct accfg_de=
vice *device,
        jobj =3D json_object_new_int(accfg_device_get_token_limit(device));
        if (!jobj)
                goto err;
-       json_object_object_add(jdevice, "token_limit", jobj);
+       if (accfg_device_get_type(device) !=3D ACCFG_DEVICE_IAX)
+               json_object_object_add(jdevice, "token_limit", jobj);
 =

        if (flags & UTIL_JSON_SAVE) {
                free(error);
@@ -277,7 +278,8 @@ struct json_object *util_device_to_json(struct accfg_de=
vice *device,
        jobj =3D json_object_new_int(accfg_device_get_max_tokens(device));
        if (!jobj)
                goto err;
-       json_object_object_add(jdevice, "max_tokens", jobj);
+       if (accfg_device_get_type(device) !=3D ACCFG_DEVICE_IAX)
+               json_object_object_add(jdevice, "max_tokens", jobj);
 =

        ulong_val =3D accfg_device_get_max_batch_size(device);
        if (ulong_val > 0) {
@@ -353,8 +355,12 @@ struct json_object *util_wq_to_json(struct accfg_wq *w=
q,
        unsigned long size =3D ULLONG_MAX;
        enum accfg_wq_mode wq_mode;
        enum accfg_wq_state wq_state;
+       struct accfg_device *dev =3D NULL;
        int int_val;
 =

+       if (wq)
+               dev =3D accfg_wq_get_device(wq);
+
        if (!jaccfg)
                return NULL;
 =

@@ -396,8 +402,10 @@ struct json_object *util_wq_to_json(struct accfg_wq *w=
q,
        }
 =

        jobj =3D json_object_new_int(accfg_wq_get_block_on_fault(wq));
-       if (jobj)
-               json_object_object_add(jaccfg, "block_on_fault", jobj);
+       if (jobj) {
+               if (accfg_device_get_type(dev) !=3D ACCFG_DEVICE_IAX)
+                       json_object_object_add(jaccfg, "block_on_fault", jobj);
+       }
 =

        jobj =3D json_object_new_int(accfg_wq_get_max_batch_size(wq));
        if (jobj)
-- =

2.26.2

--===============1184715646109867569==--
""".strip(),
        # expected
        "expected": [
            """---
 accfg/config.c          | 10 ++++++++++
 accfg/lib/libaccfg.c    |  3 +++
 accfg/libaccel_config.h |  1 +
 accfg/list.c            | 31 +++++++++++++++++++------------
 util/json.c             | 16 ++++++++++++----
 5 files changed, 45 insertions(+), 16 deletions(-)

diff --git a/accfg/config.c b/accfg/config.c
index 3887873..d36f1d9 100644
--- a/accfg/config.c
+++ b/accfg/config.c
@@ -296,6 +296,10 @@ static int group_json_set_val(struct accfg_group *grou=
p,
                json_object *jobj, char *key)
 {
        int rc, i;
+       struct accfg_device *dev =3D NULL;
+
+       if (group)
+               dev =3D accfg_group_get_device(group);
 =

        if (!group || !jobj || !key)
                return -EINVAL;
@@ -311,6 +315,12 @@ static int group_json_set_val(struct accfg_group *grou=
p,
                                                || (val < 0))
                                        return -EINVAL;
 =

+                               if ((accfg_device_get_type(dev) =3D=3D ACCFG_DEVICE_IAX)
+                                       && ((!strcmp(group_table[i].name, "tokens_reserved"))
+                                       || (!strcmp(group_table[i].name, "use_token_limit"))
+                                       || (!strcmp(group_table[i].name, "tokens_allowed")))) {
+                                       return 0;
+                               }
                                if (group_table[i].is_writable &&
                                        !group_table[i].is_writable(group,
                                                val))
diff --git a/accfg/lib/libaccfg.c b/accfg/lib/libaccfg.c
index 70553f7..cd9da42 100644
--- a/accfg/lib/libaccfg.c
+++ b/accfg/lib/libaccfg.c
@@ -39,6 +39,7 @@ static int filename_prefix_len;
 =

 ACCFG_EXPORT char *accfg_basenames[] =3D {
         [ACCFG_DEVICE_DSA]      =3D "dsa",
+       [ACCFG_DEVICE_IAX]      =3D "iax",
        NULL
 };
 =

@@ -397,6 +398,8 @@ static int device_parse_type(struct accfg_device *devic=
e)
 =

        if (!strcmp(device->device_type_str, "dsa"))
                device->type =3D ACCFG_DEVICE_DSA;
+       else if (!strcmp(device->device_type_str, "iax"))
+               device->type =3D ACCFG_DEVICE_IAX;
        else
                device->type =3D ACCFG_DEVICE_TYPE_UNKNOWN;
 =

diff --git a/accfg/libaccel_config.h b/accfg/libaccel_config.h
index 392188e..f85670c 100644
--- a/accfg/libaccel_config.h
+++ b/accfg/libaccel_config.h
@@ -31,6 +31,7 @@ extern "C" {
 /* no need to save device state */
 enum accfg_device_type {
        ACCFG_DEVICE_DSA =3D 0,
+       ACCFG_DEVICE_IAX =3D 1,
        ACCFG_DEVICE_TYPE_UNKNOWN =3D -1,
 };
 =

diff --git a/accfg/list.c b/accfg/list.c
index dfaac1f..c22da41 100644
--- a/accfg/list.c
+++ b/accfg/list.c
@@ -54,6 +54,10 @@ static struct json_object *group_to_json(struct accfg_gr=
oup *group,
 {
        struct json_object *jgroup =3D json_object_new_object();
        struct json_object *jobj =3D NULL;
+       struct accfg_device *dev =3D NULL;
+
+       if (group)
+               dev =3D accfg_group_get_device(group);
 =

        if (!jgroup)
                return NULL;
@@ -67,20 +71,23 @@ static struct json_object *group_to_json(struct accfg_g=
roup *group,
        if (!jobj)
                goto err;
 =

-       json_object_object_add(jgroup, "tokens_reserved", jobj);
-       jobj =3D json_object_new_int(accfg_group_get_use_token_limit(group));
-       if (!jobj)
-               goto err;
+       if (accfg_device_get_type(dev) !=3D ACCFG_DEVICE_IAX) {
+               json_object_object_add(jgroup, "tokens_reserved", jobj);
+               jobj =3D json_object_new_int(accfg_group_get_use_token_limit(group));
+               if (!jobj)
+                       goto err;
 =

-       json_object_object_add(jgroup, "use_token_limit", jobj);
-       jobj =3D json_object_new_int(accfg_group_get_tokens_allowed(group));
-       if (!jobj)
-               goto err;
+               json_object_object_add(jgroup, "use_token_limit", jobj);
+               jobj =3D json_object_new_int(accfg_group_get_tokens_allowed(group));
+               if (!jobj)
+                       goto err;
 =

-       json_object_object_add(jgroup, "tokens_allowed", jobj);
-       jobj =3D json_object_new_int(accfg_group_get_traffic_class_a(group));
-       if (!jobj)
-               goto err;
+               json_object_object_add(jgroup, "tokens_allowed", jobj);
+               jobj =3D json_object_new_int(accfg_group_get_traffic_class_a(
+                               group));
+               if (!jobj)
+                       goto err;
+       }
 =

        json_object_object_add(jgroup, "traffic_class_a", jobj);
        jobj =3D json_object_new_int(accfg_group_get_traffic_class_b(
diff --git a/util/json.c b/util/json.c
index 3bf8950..bb3ee88 100644
--- a/util/json.c
+++ b/util/json.c
@@ -178,7 +178,8 @@ struct json_object *util_device_to_json(struct accfg_de=
vice *device,
        jobj =3D json_object_new_int(accfg_device_get_token_limit(device));
        if (!jobj)
                goto err;
-       json_object_object_add(jdevice, "token_limit", jobj);
+       if (accfg_device_get_type(device) !=3D ACCFG_DEVICE_IAX)
+               json_object_object_add(jdevice, "token_limit", jobj);
 =

        if (flags & UTIL_JSON_SAVE) {
                free(error);
@@ -277,7 +278,8 @@ struct json_object *util_device_to_json(struct accfg_de=
vice *device,
        jobj =3D json_object_new_int(accfg_device_get_max_tokens(device));
        if (!jobj)
                goto err;
-       json_object_object_add(jdevice, "max_tokens", jobj);
+       if (accfg_device_get_type(device) !=3D ACCFG_DEVICE_IAX)
+               json_object_object_add(jdevice, "max_tokens", jobj);
 =

        ulong_val =3D accfg_device_get_max_batch_size(device);
        if (ulong_val > 0) {
@@ -353,8 +355,12 @@ struct json_object *util_wq_to_json(struct accfg_wq *w=
q,
        unsigned long size =3D ULLONG_MAX;
        enum accfg_wq_mode wq_mode;
        enum accfg_wq_state wq_state;
+       struct accfg_device *dev =3D NULL;
        int int_val;
 =

+       if (wq)
+               dev =3D accfg_wq_get_device(wq);
+
        if (!jaccfg)
                return NULL;
 =

@@ -396,8 +402,10 @@ struct json_object *util_wq_to_json(struct accfg_wq *w=
q,
        }
 =

        jobj =3D json_object_new_int(accfg_wq_get_block_on_fault(wq));
-       if (jobj)
-               json_object_object_add(jaccfg, "block_on_fault", jobj);
+       if (jobj) {
+               if (accfg_device_get_type(dev) !=3D ACCFG_DEVICE_IAX)
+                       json_object_object_add(jaccfg, "block_on_fault", jobj);
+       }
 =

        jobj =3D json_object_new_int(accfg_wq_get_max_batch_size(wq));
        if (jobj)
-- =

2.26.2
""".strip(),
        ],
    },
    # Another case
    {
        "mail": """* Saravana Kannan <saravanak@google.com> [220701 08:21]:
> On Fri, Jul 1, 2022 at 1:10 AM Saravana Kannan <saravanak@google.com> wrote:
> >
> > On Thu, Jun 30, 2022 at 11:12 PM Tony Lindgren <tony@atomide.com> wrote:
> > >
> > > * Tony Lindgren <tony@atomide.com> [220701 08:33]:
> > > > * Saravana Kannan <saravanak@google.com> [220630 23:25]:
> > > > > On Thu, Jun 30, 2022 at 4:26 PM Rob Herring <robh@kernel.org> wrote:
> > > > > >
> > > > > > On Thu, Jun 30, 2022 at 5:11 PM Saravana Kannan <saravanak@google.com> wrote:
> > > > > > >
> > > > > > > On Mon, Jun 27, 2022 at 2:10 AM Tony Lindgren <tony@atomide.com> wrote:
> > > > > > > >
> > > > > > > > * Saravana Kannan <saravanak@google.com> [220623 08:17]:
> > > > > > > > > On Thu, Jun 23, 2022 at 12:01 AM Tony Lindgren <tony@atomide.com> wrote:
> > > > > > > > > >
> > > > > > > > > > * Saravana Kannan <saravanak@google.com> [220622 19:05]:
> > > > > > > > > > > On Tue, Jun 21, 2022 at 9:59 PM Tony Lindgren <tony@atomide.com> wrote:
> > > > > > > > > > > > This issue is no directly related fw_devlink. It is a side effect of
> > > > > > > > > > > > removing driver_deferred_probe_check_state(). We no longer return
> > > > > > > > > > > > -EPROBE_DEFER at the end of driver_deferred_probe_check_state().
> > > > > > > > > > >
> > > > > > > > > > > Yes, I understand the issue. But driver_deferred_probe_check_state()
> > > > > > > > > > > was deleted because fw_devlink=on should have short circuited the
> > > > > > > > > > > probe attempt with an  -EPROBE_DEFER before reaching the bus/driver
> > > > > > > > > > > probe function and hitting this -ENOENT failure. That's why I was
> > > > > > > > > > > asking the other questions.
> > > > > > > > > >
> > > > > > > > > > OK. So where is the -EPROBE_DEFER supposed to happen without
> > > > > > > > > > driver_deferred_probe_check_state() then?
> > > > > > > > >
> > > > > > > > > device_links_check_suppliers() call inside really_probe() would short
> > > > > > > > > circuit and return an -EPROBE_DEFER if the device links are created as
> > > > > > > > > expected.
> > > > > > > >
> > > > > > > > OK
> > > > > > > >
> > > > > > > > > > Hmm so I'm not seeing any supplier for the top level ocp device in
> > > > > > > > > > the booting case without your patches. I see the suppliers for the
> > > > > > > > > > ocp child device instances only.
> > > > > > > > >
> > > > > > > > > Hmmm... this is strange (that the device link isn't there), but this
> > > > > > > > > is what I suspected.
> > > > > > > >
> > > > > > > > Yup, maybe it's because of the supplier being a device in the child
> > > > > > > > interconnect for the ocp.
> > > > > > >
> > > > > > > Ugh... yeah, this is why the normal (not SYNC_STATE_ONLY) device link
> > > > > > > isn't being created.
> > > > > > >
> > > > > > > So the aggregated view is something like (I had to set tabs = 4 space
> > > > > > > to fit it within 80 cols):
> > > > > > >
> > > > > > >     ocp: ocp {         <========================= Consumer
> > > > > > >         compatible = "simple-pm-bus";
> > > > > > >         power-domains = <&prm_per>; <=========== Supplier ref
> > > > > > >
> > > > > > >                 l4_wkup: interconnect@44c00000 {
> > > > > > >             compatible = "ti,am33xx-l4-wkup", "simple-pm-bus";
> > > > > > >
> > > > > > >             segment@200000 {  /* 0x44e00000 */
> > > > > > >                 compatible = "simple-pm-bus";
> > > > > > >
> > > > > > >                 target-module@0 { /* 0x44e00000, ap 8 58.0 */
> > > > > > >                     compatible = "ti,sysc-omap4", "ti,sysc";
> > > > > > >
> > > > > > >                     prcm: prcm@0 {
> > > > > > >                         compatible = "ti,am3-prcm", "simple-bus";
> > > > > > >
> > > > > > >                         prm_per: prm@c00 { <========= Actual Supplier
> > > > > > >                             compatible = "ti,am3-prm-inst", "ti,omap-prm-inst";
> > > > > > >                         };
> > > > > > >                     };
> > > > > > >                 };
> > > > > > >             };
> > > > > > >         };
> > > > > > >     };
> > > > > > >
> > > > > > > The power-domain supplier is the great-great-great-grand-child of the
> > > > > > > consumer. It's not clear to me how this is valid. What does it even
> > > > > > > mean?
> > > > > > >
> > > > > > > Rob, is this considered a valid DT?
> > > > > >
> > > > > > Valid DT for broken h/w.
> > > > >
> > > > > I'm not sure even in that case it's valid. When the parent device is
> > > > > in reset (when the SoC is coming out of reset), there's no way the
> > > > > descendant is functional. And if the descendant is not functional, how
> > > > > is the parent device powered up? This just feels like an incorrect
> > > > > representation of the real h/w.
> > > >
> > > > It should be correct representation based on scanning the interconnects
> > > > and looking at the documentation. Some interconnect parts are wired
> > > > always-on and some interconnect instances may be dual-mapped.
> >
> > Thanks for helping to debug this. Appreciate it.
> >
> > > >
> > > > We have a quirk to probe prm/prcm first with pdata_quirks_init_clocks().
> >
> > :'(
> >
> > I checked out the code. These prm devices just get populated with NULL
> > as the parent. So they are effectively top level devices from the
> > perspective of driver core.
> >
> > > > Maybe that also now fails in addition to the top level interconnect
> > > > probing no longer producing -EPROBE_DEFER.
> >
> > As far as I can tell pdata_quirks_init_clocks() is just adding these
> > prm devices (amongst other drivers). So I don't expect that to fail.
> >
> > > >
> > > > > > So the domain must be default on and then simple-pm-bus is going to
> > > > > > hold a reference to the domain preventing it from ever getting powered
> > > > > > off and things seem to work. Except what happens during suspend?
> > > > >
> > > > > But how can simple-pm-bus even get a reference? The PM domain can't
> > > > > get added until we are well into the probe of the simple-pm-bus and
> > > > > AFAICT the genpd attach is done before the driver probe is even
> > > > > called.
> > > >
> > > > The prm/prcm gets of_platform_populate() called on it early.
> >
> > :'(
> >
> > > The hackish patch below makes things boot for me, not convinced this
> > > is the preferred fix compared to earlier deferred probe handling though.
> > > Going back to the init level tinkering seems like a step back to me.
> >
> > The goal of fw_devlink is to avoid init level tinkering and it does
> > help with that in general. But these kinds of quirks are going to need
> > a few exceptions -- with them being quirks and all. And this change
> > will avoid an unnecessary deferred probe (that used to happen even
> > before my change).
> >
> > The other option to handle this quirk is to create the invalid
> > (consumer is parent of supplier) fwnode_link between the prm device
> > and its consumers when the prm device is populated. Then fw_devlink
> > will end up creating a device link when ocp gets added. But I'm not
> > sure if it's going to be easy to find and add all those consumers.
> >
> > I'd say, for now, let's go with this patch below. I'll see if I can
> > get fw_devlink to handle these odd quirks without breaking the normal
> > cases or making them significantly slower. But that'll take some time
> > and I'm not sure there'll be a nice solution.
> 
> Can you check if this hack helps? If so, then I can think about
> whether we can pick it up without breaking everything else. Copy-paste
> tab mess up warning.

Yeah so manually applying your patch while updating it against
next-20220624 kernel boots for me. I ended up with the following
changes FYI.

Also, looks like both with the initcall change for prm, and the patch
below, there seems to be also another problem where my test devices no
longer properly idle somehow compared to reverting the your two patches
in next.

Regards,

Tony

8< -------------------
diff --git a/drivers/of/property.c b/drivers/of/property.c
--- a/drivers/of/property.c
+++ b/drivers/of/property.c
@@ -1138,18 +1138,6 @@ static int of_link_to_phandle(struct device_node *con_np,
                return -ENODEV;
        }
 
-       /*
-        * Don't allow linking a device node as a consumer of one of its
-        * descendant nodes. By definition, a child node can't be a functional
-        * dependency for the parent node.
-        */
-       if (of_is_ancestor_of(con_np, sup_np)) {
-               pr_debug("Not linking %pOFP to %pOFP - is descendant\n",
-                        con_np, sup_np);
-               of_node_put(sup_np);
-               return -EINVAL;
-       }
-
        /*
         * Don't create links to "early devices" that won't have struct devices
         * created for them.
@@ -1163,9 +1151,27 @@ static int of_link_to_phandle(struct device_node *con_np,
                of_node_put(sup_np);
                return -ENODEV;
        }
-       put_device(sup_dev);
+
+       /*
+        * Don't allow linking a device node as a consumer of one of its
+        * descendant nodes. By definition, a child node can't be a functional
+        * dependency for the parent node.
+        *
+        * However, if the child node already has a device while the parent is
+        * in the process of being added, it's probably some weird quirk
+        * handling. So, don't both checking if the consumer is an ancestor of
+        * the supplier.
+        */
+       if (!sup_dev && of_is_ancestor_of(con_np, sup_np)) {
+               pr_debug("Not linking %pOFP to %pOFP - is descendant\n",
+                        con_np, sup_np);
+               put_device(sup_dev);
+               of_node_put(sup_np);
+               return -EINVAL;
+       }
 
        fwnode_link_add(of_fwnode_handle(con_np), of_fwnode_handle(sup_np));
+       put_device(sup_dev);
        of_node_put(sup_np);
 
        return 0;
-- 
2.36.1
_______________________________________________
iommu mailing list
iommu@lists.linux-foundation.org
""",
        "expected": [
            """diff --git a/drivers/of/property.c b/drivers/of/property.c
--- a/drivers/of/property.c
+++ b/drivers/of/property.c
@@ -1138,18 +1138,6 @@ static int of_link_to_phandle(struct device_node *con_np,
                return -ENODEV;
        }
 
-       /*
-        * Don't allow linking a device node as a consumer of one of its
-        * descendant nodes. By definition, a child node can't be a functional
-        * dependency for the parent node.
-        */
-       if (of_is_ancestor_of(con_np, sup_np)) {
-               pr_debug("Not linking %pOFP to %pOFP - is descendant\n",
-                        con_np, sup_np);
-               of_node_put(sup_np);
-               return -EINVAL;
-       }
-
        /*
         * Don't create links to "early devices" that won't have struct devices
         * created for them.
@@ -1163,9 +1151,27 @@ static int of_link_to_phandle(struct device_node *con_np,
                of_node_put(sup_np);
                return -ENODEV;
        }
-       put_device(sup_dev);
+
+       /*
+        * Don't allow linking a device node as a consumer of one of its
+        * descendant nodes. By definition, a child node can't be a functional
+        * dependency for the parent node.
+        *
+        * However, if the child node already has a device while the parent is
+        * in the process of being added, it's probably some weird quirk
+        * handling. So, don't both checking if the consumer is an ancestor of
+        * the supplier.
+        */
+       if (!sup_dev && of_is_ancestor_of(con_np, sup_np)) {
+               pr_debug("Not linking %pOFP to %pOFP - is descendant\n",
+                        con_np, sup_np);
+               put_device(sup_dev);
+               of_node_put(sup_np);
+               return -EINVAL;
+       }
 
        fwnode_link_add(of_fwnode_handle(con_np), of_fwnode_handle(sup_np));
+       put_device(sup_dev);
        of_node_put(sup_np);
 
        return 0;
-- 
2.36.1"""
        ],
    },
]


def test_real_mails() -> None:
    i = 0
    for case in real_mails:
        output = extract_patches(case["mail"])
        # i += 1
        # if i == 2:
        #     continue
        # assert len(output) == 1

        assert output == case["expected"]
