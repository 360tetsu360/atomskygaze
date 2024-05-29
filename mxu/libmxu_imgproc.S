	.file	1 "mxu_imgproc.c"
	.section .mdebug.abi32
	.previous
	.gnu_attribute 4, 1
	.gnu_attribute 8, 1
	.abicalls
	.text
	.align	2
	.globl	buffer_diff
	.set	nomips16
	.set	nomicromips
	.ent	buffer_diff
	.type	buffer_diff, @function
buffer_diff:
	.frame	$sp,0,$31		# vars= 0, regs= 0/0, args= 0, gp= 0
	.mask	0x00000000,0
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	beq	$7,$0,$L8
	move	$2,$0

$L5:
	lu1qx	$vr4,$2($4)
	lu1qx	$vr0,$2($5)
	subb	$vr0,$vr4,$vr0
	su1qx	$vr0,$2($6)
	addiu	$2,$2,16
	sltu	$3,$2,$7
	bne	$3,$0,$L5
	nop

$L8:
	j	$31
	nop

	.set	macro
	.set	reorder
	.end	buffer_diff
	.size	buffer_diff, .-buffer_diff
	.align	2
	.globl	buffer_absdiff
	.set	nomips16
	.set	nomicromips
	.ent	buffer_absdiff
	.type	buffer_absdiff, @function
buffer_absdiff:
	.frame	$sp,0,$31		# vars= 0, regs= 0/0, args= 0, gp= 0
	.mask	0x00000000,0
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	beq	$7,$0,$L15
	move	$2,$0

$L13:
	lu1qx	$vr0,$2($4)
	lu1qx	$vr4,$2($5)
	maxub	$vr8,$vr0,$vr4
	minub	$vr0,$vr0,$vr4
	subuub	$vr0,$vr8,$vr0
	su1qx	$vr0,$2($6)
	addiu	$2,$2,16
	sltu	$3,$2,$7
	bne	$3,$0,$L13
	nop

$L15:
	j	$31
	nop

	.set	macro
	.set	reorder
	.end	buffer_absdiff
	.size	buffer_absdiff, .-buffer_absdiff
	.align	2
	.globl	buffer_add
	.set	nomips16
	.set	nomicromips
	.ent	buffer_add
	.type	buffer_add, @function
buffer_add:
	.frame	$sp,0,$31		# vars= 0, regs= 0/0, args= 0, gp= 0
	.mask	0x00000000,0
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	beq	$7,$0,$L22
	move	$2,$0

$L20:
	lu1qx	$vr4,$2($4)
	lu1qx	$vr0,$2($5)
	adduub	$vr0,$vr4,$vr0
	su1qx	$vr0,$2($6)
	addiu	$2,$2,16
	sltu	$3,$2,$7
	bne	$3,$0,$L20
	nop

$L22:
	j	$31
	nop

	.set	macro
	.set	reorder
	.end	buffer_add
	.size	buffer_add, .-buffer_add
	.align	2
	.globl	buffer_div_add
	.set	nomips16
	.set	nomicromips
	.ent	buffer_div_add
	.type	buffer_div_add, @function
buffer_div_add:
	.frame	$sp,24,$31		# vars= 16, regs= 0/0, args= 0, gp= 8
	.mask	0x00000000,0
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	li	$8,16842752			# 0x1010000
	addiu	$8,$8,257
	andi	$6,$6,0x00ff
	multu	$6,$8
	addiu	$sp,$sp,-24
	mfhi	$3
	mflo	$2
	sw	$2,8($sp)
	mul	$12,$6,$8
	mtlo	$2
	addu	$11,$12,$3
	sw	$11,12($sp)
	mthi	$3
	sw	$11,20($sp)
	mflo	$8
	sw	$8,16($sp)
	lu1q	$vr12,8($sp)
	beq	$7,$0,$L23
	move	$10,$0

	lib	$vr4,0
$L36:
	beq	$6,$0,$L25
	orv	$vr8,$vr4,$vr4

	move	$3,$4
	move	$2,$0
$L26:
	lw	$9,0($3)
	addiu	$2,$2,1
	sltu	$8,$2,$6
	lu1qx	$vr0,$10($9)
	addiu	$3,$3,4
	bnez16b	$vr12,1f
	divub	$vr16,$vr0,$vr12
	break	7
1:
	bnez16b	$vr12,1f
	modub	$vr0,$vr0,$vr12
	break	7
1:
	adduub	$vr4,$vr16,$vr4
	bne	$8,$0,$L26
	adduub	$vr8,$vr0,$vr8

$L25:
	bnez16b	$vr12,1f
	divub	$vr8,$vr8,$vr12
	break	7
1:
	adduub	$vr4,$vr8,$vr4
	su1qx	$vr4,$10($5)
	addiu	$10,$10,16
	sltu	$2,$10,$7
	bne	$2,$0,$L36
	lib	$vr4,0

$L23:
	j	$31
	addiu	$sp,$sp,24

	.set	macro
	.set	reorder
	.end	buffer_div_add
	.size	buffer_div_add, .-buffer_div_add
	.align	2
	.globl	fast_memcpy
	.set	nomips16
	.set	nomicromips
	.ent	fast_memcpy
	.type	fast_memcpy, @function
fast_memcpy:
	.frame	$sp,0,$31		# vars= 0, regs= 0/0, args= 0, gp= 0
	.mask	0x00000000,0
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	beq	$6,$0,$L43
	move	$2,$0

$L41:
	lu1qx	$vr0,$2($4)
	su1qx	$vr0,$2($5)
	addiu	$2,$2,16
	sltu	$3,$2,$6
	bne	$3,$0,$L41
	nop

$L43:
	j	$31
	nop

	.set	macro
	.set	reorder
	.end	fast_memcpy
	.size	fast_memcpy, .-fast_memcpy
	.align	2
	.globl	fast_memset
	.set	nomips16
	.set	nomicromips
	.ent	fast_memset
	.type	fast_memset, @function
fast_memset:
	.frame	$sp,0,$31		# vars= 0, regs= 0/0, args= 0, gp= 0
	.mask	0x00000000,0
	.fmask	0x00000000,0
	.set	noreorder
	.set	nomacro
	beq	$6,$0,$L51
	move	$2,$0

	lib	$vr0,0
	addu	$5,$4,$2
$L50:
	addiu	$2,$2,16
	sltu	$3,$2,$6
	su1q	$vr0,0($5)
	bne	$3,$0,$L50
	addu	$5,$4,$2

$L51:
	j	$31
	nop

	.set	macro
	.set	reorder
	.end	fast_memset
	.size	fast_memset, .-fast_memset
	.ident	"GCC: (Ingenic r2.3.3 2016.12) 4.7.2"
