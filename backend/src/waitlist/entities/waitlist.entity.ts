import { Entity, PrimaryGeneratedColumn, Column, CreateDateColumn } from 'typeorm';

@Entity('waitlist')
export class WaitlistEntry {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  email: string;

  @Column({ unique: true, nullable: true })
  username: string;

  @Column({ nullable: true })
  businessName: string;

  @Column({ nullable: true })
  country: string;

  @Column({ default: 0 })
  referrals: number;

  @Column({ nullable: true })
  referralCode: string;

  @Column({ default: false })
  unsubscribed: boolean;

  /** Secure token used for one-click unsubscribe and deletion (no login required) */
  @Column({ nullable: true })
  unsubscribeToken: string;

  @Column({ default: false })
  notified: boolean;

  @CreateDateColumn()
  createdAt: Date;
}
