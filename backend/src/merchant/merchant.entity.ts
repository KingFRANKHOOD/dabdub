import { Column, CreateDateColumn, Entity, PrimaryGeneratedColumn } from 'typeorm';

@Entity('merchants')
export class Merchant {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  email: string;

  @Column()
  passwordHash: string;

  @Column({ nullable: true })
  businessName: string;

  @Column({ default: false })
  emailVerified: boolean;

  @Column({ nullable: true, type: 'varchar' })
  emailVerifyToken: string | null;

  @Column({ nullable: true, type: 'timestamptz' })
  emailVerifyExpiry: Date | null;

  @CreateDateColumn()
  createdAt: Date;
}
