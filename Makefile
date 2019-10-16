NAME		=	koop

ARCH		=	x86_64

RUST_LIB	=	target/$(NAME)/debug/libkoop.a

KERNEL		=	$(NAME)-$(ARCH).bin

ISO			=	$(NAME)-$(ARCH).iso

ASM			=	multiboot_header.asm	\
				boot.asm				\
				long_mode_init.asm

LD_SCRIPT	=	linker.ld

GRUB_CFG	=	grub.cfg

SRCDIR		=	src
ASMDIR		=	$(SRCDIR)/arch/$(ARCH)
GRUBDIR		=	$(SRCDIR)/grub

BUILDDIR	=	build
OBJDIR		=	$(BUILDDIR)/obj
KERNELDIR	=	$(BUILDDIR)/target/$(ARCH)

ASM			:=	$(addprefix $(ASMDIR)/, $(ASM))
LD_SCRIPT	:=	$(addprefix $(ASMDIR)/, $(LD_SCRIPT))
KERNEL		:=	$(addprefix $(KERNELDIR)/, $(KERNEL))
ISO			:=	$(addprefix $(KERNELDIR)/, $(ISO))
OBJ			:=	$(subst $(ASMDIR), $(OBJDIR), $(ASM:.asm=.o))

AS			=	nasm
LD			=	ld
RM			=	rm -rf

RELEASE		=

QEMU_OPT	=	-m 2G

all:		$(ISO)

$(KERNEL): 	$(OBJ) $(LD_SCRIPT) cargo
	mkdir -p $(KERNELDIR)
	$(LD) -n -T $(LD_SCRIPT) -o $(KERNEL) $(OBJ) $(RUST_LIB)
	$(STRIP_DEBUG)

cargo:
	cargo-fmt --all
	cargo +nightly xbuild $(RELEASE) --target $(ASMDIR)/koop.json

$(ISO):		$(KERNEL) $(GRUBDIR)/$(GRUB_CFG)
	mkdir -p $(BUILDDIR)/iso/boot/grub
	cp $(KERNEL) $(BUILDDIR)/iso/boot/kernel.bin
	cp $(GRUBDIR)/$(GRUB_CFG) $(BUILDDIR)/iso/boot/grub/grub.cfg
	grub-mkrescue -o $(ISO) $(BUILDDIR)/iso

run:
	qemu-system-x86_64 -smp 2 -cdrom $(ISO) $(QEMU_OPT) -d int -no-reboot

run-kvm:
	qemu-system-x86_64 -smp 2 --enable-kvm -cpu host -cdrom $(ISO) $(QEMU_OPT) -no-reboot

riso: RELEASE=--release
riso: RUST_LIB=target/$(NAME)/release/libkoop.a
riso: STRIP_DEBUG=strip $(KERNEL)
riso:	$(ISO)

run-debug:	$(ISO)
	qemu-system-x86_64 -smp 2 -cdrom $(ISO) $(QEMU_OPT) -s -S -d int -no-reboot

iso:	$(ISO)

$(OBJDIR)/%.o:	$(ASMDIR)/%.asm | $(OBJDIR)
	nasm -felf64 $< -o $@

$(OBJDIR):
	mkdir -p $(OBJDIR)

clean:
	$(RM) $(BUILDDIR)
	cargo clean

re:	clean $(NAME)

.PHONY: clean re
