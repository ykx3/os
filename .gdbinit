source ~/.gef-2024.01.py
file esp/KERNEL.ELF
gef config context.layout "-legend regs -stack code -args source -threads -trace extra memory"
gef-remote localhost 1234
tmux-setup
b ysos_kernel::init
