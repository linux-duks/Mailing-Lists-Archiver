# LKML5Ws: The What, When, Who, Where, and Why in the Linux Kernel Mailing Lists - A Columnar Dataset

The Linux kernel, like other large and long-lived Free Software projects, utilizes mailing lists as the traditional medium for all development, bug reporting, and pivotal discussions on the project's future. However, as a consequence of the decentralized development model used in the Linux kernel, these emails are spread over hundreds of different mailing lists, with different communities and code maintainership models. This paper presents the LKML5Ws dataset. With over 20 million emails from 345 different mailing lists, our massive relational dataset provides a comprehensive overview of the last 20 years of Linux kernel development. Beyond shedding light on the awe-inspiring number of patches, discussions, and contributors involved in the project, our dataset serves as a basis for those interested in studying the intricate and knowledge-dense nature of the Linux kernel development process.

<https://github.com/linux-duks/Mailing-Lists-Archiver>

## Using this Dataset

The dataset is an Apache Parquet dataset partitioned by each mailing list.

To use this dataset, it must first be uncompressed using the attached [decompression_script.sh](decompression_script.sh).

Open a terminal in the dataset folder (if not already, `cd dataset`).

run `bash ./decompression_script.sh`

After this, a `LKML5Ws` should be present in the current directory with all partitions decompressed.

Analyses can be performed targeting a specific partition, such as `list=dev.linux.lists.virtualization`, or with all partitions.

### Example Analyses

In our repository, which contains the software used to create this dataset, we also provide scripts that we used to develop example analyses.

- Notebook Example: [Mailing-Lists-Archiver/analysis/src/experimenting.ipynb](https://archive.softwareheritage.org/swh:1:cnt:e484d0a379b9e3da8f4e3acd696cb4c324682f65;origin=https://github.com/linux-duks/Mailing-Lists-Archiver;anchor=swh:1:rev:133559afe421926d6579370a90fee329fd4d518d;path=/analysis/src/experimenting.ipynb)
- Script Example: [Mailing-Lists-Archiver/blob/analysis/analysis/src/make_analysis.py](https://archive.softwareheritage.org/swh:1:cnt:b8cab9d1dd1e75e5a1219086bd5cdf8bdfe77943;origin=https://github.com/linux-duks/Mailing-Lists-Archiver;anchor=swh:1:rev:133559afe421926d6579370a90fee329fd4d518d;path=/analysis/src/make_analysis.py)
