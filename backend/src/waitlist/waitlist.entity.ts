import { Column, CreateDateColumn, Entity, PrimaryGeneratedColumn } from 'typeorm';

@Entity('waitlist')
export class WaitlistEntry {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  email: string;

  @Column({ nullable: true })
  username: string;

  @Column({ nullable: true })
  businessName: string;

  @Column({ nullable: true })
  country: string;

  /** Unique referral code this member can share */
  @Column({ unique: true })
  referralCode: string;

  /** Number of successful referrals made by this member */
  @Column({ default: 0 })
  referralCount: number;

  /** Queue position — lower = earlier */
  @Column()
  position: number;

  @CreateDateColumn()
  createdAt: Date;
}
