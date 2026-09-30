import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  DeleteDateColumn,
} from 'typeorm';
import { Exclude, Transform } from 'class-transformer';

export enum MerchantStatus {
  ACTIVE = 'active',
  SUSPENDED = 'suspended',
  PENDING = 'pending',
}

export enum MerchantRole {
  ADMIN = 'admin',
  MERCHANT = 'merchant',
  SUPERADMIN = 'superadmin',
}

@Entity('merchants')
export class Merchant {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  email: string;

  @Exclude()
  @Column()
  passwordHash: string;

  @Column()
  businessName: string;

  @Column({ nullable: true })
  businessType: string;

  @Column({ nullable: true })
  country: string;

  @Column({ type: 'enum', enum: MerchantStatus, default: MerchantStatus.PENDING })
  status: MerchantStatus;

  @Column({ type: 'enum', enum: MerchantRole, default: MerchantRole.MERCHANT })
  role: MerchantRole;

  @Column({ nullable: true })
  apiKey: string;

  @Exclude()
  @Column({ nullable: true })
  apiKeyHash: string;

  @Column({ type: 'decimal', precision: 5, scale: 4, default: 0.015 })
  feeRate: number;

  @Column({ default: false })
  sandboxMode: boolean;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  @DeleteDateColumn()
  deletedAt: Date;
}
