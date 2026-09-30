import { Column, Entity, ManyToOne, PrimaryGeneratedColumn } from 'typeorm';
import { Merchant } from '../merchant/merchant.entity';

@Entity('refresh_tokens')
export class RefreshToken {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  tokenHash: string;

  @Column()
  merchantId: string;

  @ManyToOne(() => Merchant, { onDelete: 'CASCADE' })
  merchant: Merchant;

  @Column()
  expiresAt: Date;
}
